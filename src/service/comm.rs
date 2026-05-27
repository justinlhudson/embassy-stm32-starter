//! Application-level message protocol layered on top of HDLC framing.
//!
//! Wire format (little-endian, header = 9 bytes):
//! ```text
//! ┌─────────┬─────┬───────────┬──────────┬────────┬─────────────┐
//! │ Command │ Id  │ Fragments │ Fragment │ Length │   Payload   │
//! │  (u16)  │(u8) │   (u16)   │  (u16)   │ (u16)  │ (0..=256 B) │
//! └─────────┴─────┴───────────┴──────────┴────────┴─────────────┘
//! ```
//!
//! Note: `Length` in the wire frame is the on-the-wire payload length and
//! must equal `payload.len()` in the [`Message`] struct.

use crate::hardware::serial;
use crate::protocol::hdlc;
use core::sync::atomic::{AtomicU8, Ordering};
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;
use num_enum::TryFromPrimitive;

// ---- sizing --------------------------------------------------------------
pub const COMMS_HEADER_LEN: usize = 9;
pub const COMMS_MAX_PAYLOAD: usize = 256;
const COMMS_QUEUE_DEPTH: usize = 3;
const RX_BUF_SIZE: usize = 512;

pub type CommsPayload = Vec<u8, COMMS_MAX_PAYLOAD>;
pub type FramedBuf = Vec<u8, RX_BUF_SIZE>;
type RxBuf = Vec<u8, RX_BUF_SIZE>;
type DecodedBuf = Vec<u8, RX_BUF_SIZE>;

// ---- FCS error counter ---------------------------------------------------
static FCS_ERROR_COUNT: AtomicU8 = AtomicU8::new(0);
pub fn fcs_error_count() -> u8 { FCS_ERROR_COUNT.load(Ordering::Relaxed) }

// ---- commands ------------------------------------------------------------
#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Command {
  Ack     = 0x01,
  Nak     = 0x02,
  Ping    = 0x03,
  Raw     = 0x04,
  Version = 0x05,
}

impl From<Command> for u16 { fn from(c: Command) -> u16 { c as u16 } }

// ---- message -------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Message {
  pub command: u16,
  pub id: u8,
  pub fragments: u16,
  pub fragment: u16,
  pub payload: CommsPayload,
}

impl Default for Message {
  fn default() -> Self {
    Self { command: 0, id: 0, fragments: 1, fragment: 0, payload: Vec::new() }
  }
}

impl Message {
  pub fn new<C: Into<u16>>(command: C, payload: &[u8]) -> Self {
    let mut buf: CommsPayload = Vec::new();
    let take = core::cmp::min(payload.len(), COMMS_MAX_PAYLOAD);
    let _ = buf.extend_from_slice(&payload[..take]);
    Self { command: command.into(), id: 0, fragments: 1, fragment: 1, payload: buf }
  }

  /// `length` reported on the wire (always equals `payload.len()`).
  pub fn length(&self) -> u16 { self.payload.len() as u16 }
}

// ---- queue ---------------------------------------------------------------
static COMMS_MSG_QUEUE: Channel<CriticalSectionRawMutex, Message, COMMS_QUEUE_DEPTH> = Channel::new();

/// Read the next parsed message (non-blocking).
pub fn read() -> Option<Message> { COMMS_MSG_QUEUE.try_receive().ok() }

/// Await the next parsed message.
pub async fn recv() -> Message { COMMS_MSG_QUEUE.receive().await }

// ---- write ---------------------------------------------------------------
/// Encode a [`Message`], HDLC-frame it, and write to the serial.
pub fn write<W: embedded_io::Write>(serial: &mut W, msg: &Message) {
  let mut buf: Vec<u8, { COMMS_HEADER_LEN + COMMS_MAX_PAYLOAD }> = Vec::new();
  let len = msg.length();

  let _ = buf.extend_from_slice(&msg.command.to_le_bytes());
  let _ = buf.push(msg.id);
  let _ = buf.extend_from_slice(&msg.fragments.to_le_bytes());
  let _ = buf.extend_from_slice(&msg.fragment.to_le_bytes());
  let _ = buf.extend_from_slice(&len.to_le_bytes());
  let _ = buf.extend_from_slice(&msg.payload);

  let mut framed: FramedBuf = Vec::new();
  hdlc::hdlc_frame(&buf, &mut framed);
  serial::write(serial, &framed);
}

// ---- canned replies ------------------------------------------------------
/// Build a Version reply echoing the request's id; payload is `CARGO_PKG_VERSION`.
pub fn version_reply(req: &Message) -> Message {
  let mut msg = Message::new(Command::Version, env!("CARGO_PKG_VERSION").as_bytes());
  msg.id = req.id;
  msg
}

// ---- consumer task -------------------------------------------------------
/// Async task: pull raw bytes from the serial RX queue, deframe HDLC, parse
/// messages, and enqueue them for the application.
#[embassy_executor::task]
pub async fn consumer_task() {
  let mut rx_buf: RxBuf = Vec::new();
  let mut decoded: DecodedBuf = Vec::new();
  loop {
    let msg = serial::recv_raw().await;
    if rx_buf.extend_from_slice(&msg).is_err() {
      defmt::warn!("comm consumer: rx_buf overflow ({} bytes); resync", rx_buf.len());
      rx_buf.clear();
      continue;
    }

    loop {
      match hdlc::hdlc_deframe(&mut rx_buf, &mut decoded) {
        Ok(()) => {
          if let Some(m) = parse_frame(&decoded) {
            let _ = COMMS_MSG_QUEUE.try_send(m);
          }
        }
        Err(hdlc::HdlcError::Incomplete) => break,
        Err(hdlc::HdlcError::FcsMismatch { received, calculated, len }) => {
          FCS_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
          defmt::warn!(
            "HDLC FCS error: recv={=u16:x}, calc={=u16:x}, len={}",
            received, calculated, len
          );
          // Continue looking for the next frame; do NOT reset the MCU here —
          // that's a policy decision left to the application.
        }
      }
    }
  }
}

/// Convenience for binaries: spawn the consumer task. Call once after init.
pub fn start(spawner: embassy_executor::Spawner) {
  spawner.spawn(consumer_task().unwrap());
}

// ---- dispatch helper -----------------------------------------------------
/// Handle one message: built-ins (Ping echo, Version reply) are answered
/// automatically; everything else is forwarded to `user`. Returns the
/// command id (`Ok`) so callers can drive their own LED/state machine.
pub fn dispatch<F>(tx: &mut UartTx<'static, Async>, msg: &Message, mut user: F)
where
  F: FnMut(&Message, &mut UartTx<'static, Async>),
{
  use core::convert::TryFrom;
  match Command::try_from(msg.command) {
    Ok(Command::Ping) => write(tx, msg),
    Ok(Command::Version) => write(tx, &version_reply(msg)),
    _ => user(msg, tx),
  }
}

// ---- parse ---------------------------------------------------------------
fn parse_frame(bytes: &[u8]) -> Option<Message> {
  if bytes.len() < COMMS_HEADER_LEN {
    defmt::warn!("Frame too short: {} bytes", bytes.len());
    return None;
  }
  let cmd  = u16::from_le_bytes([bytes[0], bytes[1]]);
  let id   = bytes[2];
  let frags= u16::from_le_bytes([bytes[3], bytes[4]]);
  let frag = u16::from_le_bytes([bytes[5], bytes[6]]);
  let len  = u16::from_le_bytes([bytes[7], bytes[8]]) as usize;

  if bytes.len() != COMMS_HEADER_LEN + len {
    defmt::warn!("Frame length mismatch: got {}, header says {}+{}", bytes.len(), COMMS_HEADER_LEN, len);
    return None;
  }
  if len > COMMS_MAX_PAYLOAD {
    defmt::warn!("Payload {} > max {}", len, COMMS_MAX_PAYLOAD);
    return None;
  }

  let mut payload: CommsPayload = Vec::new();
  payload.extend_from_slice(&bytes[COMMS_HEADER_LEN..COMMS_HEADER_LEN + len]).ok()?;
  Some(Message { command: cmd, id, fragments: frags, fragment: frag, payload })
}

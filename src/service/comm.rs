use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;

use crate::hardware::serial;
use crate::protocol::hdlc;

// Define constants for queue depth and byte vector sizes
const COMMS_BYTE_VEC_SIZE: usize = 512;
const COMMS_QUEUE_DEPTH: usize = 3;
pub const COMMS_MAX_PAYLOAD: usize = 256; // half to account for escaping

// Byte vector aliases used throughout this module
// Allow room for larger inbound/outbound frames (escaping can ~double size)
pub type ByteVec = Vec<u8, COMMS_BYTE_VEC_SIZE>;
pub type FramedBuf = Vec<u8, COMMS_BYTE_VEC_SIZE>;
pub type CommsPayload = Vec<u8, COMMS_MAX_PAYLOAD>;
pub type CommsFrameBuf = Vec<u8, { COMMS_HEADER_LEN + COMMS_MAX_PAYLOAD }>; // COMMS_HEADER_LEN=9 now

/// Command identifiers for Comms messages.
#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Command {
  Ack = 0x01,
  Nak = 0x02,
  Ping = 0x03,
  Raw = 0x04,
}

impl From<Command> for u16 {
  fn from(c: Command) -> Self {
    c as u16
  }
}

impl core::convert::TryFrom<u16> for Command {
  type Error = ();
  fn try_from(value: u16) -> Result<Self, Self::Error> {
    match value {
      0x01 => Ok(Command::Ack),
      0x02 => Ok(Command::Nak),
      0x03 => Ok(Command::Ping),
      0x04 => Ok(Command::Raw),
      _ => Err(()),
    }
  }
}

// Comms message format (little-endian):
// - command:      u16
// - id:           u8
// - fragments:    u16 (total fragments)
// - fragment:     u16 (0-based index)
// - length:       u16  (payload length in bytes)
// - payload:      [u8; length]

pub const COMMS_HEADER_LEN: usize = 9;

#[derive(Clone, Debug)]
pub struct Message {
  pub command: u16,
  pub id: u8,         // todo: future use
  pub fragments: u16, // todo: future use
  pub fragment: u16,  // todo: future use
  pub length: u16,
  pub payload: CommsPayload,
}

impl Default for Message {
  fn default() -> Self {
    Self {
      command: 0,
      id: 0,
      fragments: 1,
      fragment: 0,
      length: 0,
      payload: Vec::new(),
    }
  }
}

impl Message {
  /// Convenience constructor with defaults (id=0, fragments=1, fragment=1).
  pub fn new<C: Into<u16>>(command: C, payload: &[u8]) -> Self {
    let mut buf: Vec<u8, COMMS_MAX_PAYLOAD> = Vec::new();
    let take = core::cmp::min(payload.len(), COMMS_MAX_PAYLOAD);
    let _ = buf.extend_from_slice(&payload[..take]);
    Self {
      command: command.into(),
      id: 0,
      fragments: 1,
      fragment: 1,
      length: take as u16,
      payload: buf,
    }
  }
}

// Queue of parsed Comms messages
static COMMS_MSG_QUEUE: Channel<CriticalSectionRawMutex, Message, COMMS_QUEUE_DEPTH> = Channel::new();

/// Encode a Message and send over HDLC
pub fn write<W: embedded_io::Write>(serial: &mut W, msg: &Message) {
  // Build unframed message (header + payload)
  let mut buf: CommsFrameBuf = Vec::new();
  let len_usize = core::cmp::min(msg.payload.len(), COMMS_MAX_PAYLOAD);
  let len: u16 = len_usize as u16; // Use actual payload length, not msg.length field

  buf.extend_from_slice(&msg.command.to_le_bytes()).ok();
  buf.push(msg.id).ok();
  buf.extend_from_slice(&msg.fragments.to_le_bytes()).ok();
  buf.extend_from_slice(&msg.fragment.to_le_bytes()).ok();
  buf.extend_from_slice(&len.to_le_bytes()).ok();

  buf.extend_from_slice(&msg.payload[..len_usize]).ok();

  // HDLC-frame and write
  let mut framed: FramedBuf = Vec::new();
  hdlc::hdlc_frame(&buf, &mut framed);
  serial::write(serial, &framed);
}

/// Async task: read bytes from serial queue, deframe, and publish decoded payloads
#[embassy_executor::task]
pub async fn serial_hdlc_consumer_task() {
  let mut rx_buf: ByteVec = Vec::new();
  let mut decoded: ByteVec = Vec::new();
  loop {
    // Wait for a new message from the serial RX queue
    let msg = serial::recv_raw().await;
    // Append to buffer
    rx_buf.extend_from_slice(&msg).ok();

    // Safety check: clear buffer if it grows too large
    if rx_buf.len() >= COMMS_BYTE_VEC_SIZE {
      defmt::warn!("serial_hdlc_consumer_task: rx_buf overflow ({} bytes), clearing buffer", rx_buf.len());
      rx_buf.clear();
    }

    // Try to decode HDLC frame(s)
    while try_decode_hdlc(&mut rx_buf, &mut decoded) {
      // Try to parse as a Comms frame and publish
      if let Some(msg) = try_parse_comms_frame(&decoded) {
        let _ = COMMS_MSG_QUEUE.try_send(msg);
      }
    }
  }
}

/// Read next parsed Comms message (non-blocking).
pub fn read() -> Option<Message> {
  COMMS_MSG_QUEUE.try_receive().ok()
}

// --- Internal helpers ---

/// Try to decode an HDLC frame from a buffer of received serial data
fn try_decode_hdlc(buf: &mut ByteVec, out: &mut ByteVec) -> bool {
  hdlc::hdlc_deframe(buf, out).is_some()
}

/// Try to parse a Comms message from a byte slice (little-endian)
fn try_parse_comms_frame(bytes: &[u8]) -> Option<Message> {
  if bytes.len() < COMMS_HEADER_LEN {
    return None;
  }
  let cmd = u16::from_le_bytes([bytes[0], bytes[1]]);
  let id = bytes[2];
  let frags = u16::from_le_bytes([bytes[3], bytes[4]]);
  let frag = u16::from_le_bytes([bytes[5], bytes[6]]);
  let len = u16::from_le_bytes([bytes[7], bytes[8]]) as usize;
  let total = COMMS_HEADER_LEN + len;

  // Check if frame has the expected length (header + payload)
  if bytes.len() != total {
    // Handle common case: extra 0x00 byte inserted after header
    if bytes.len() == total + 1 && bytes.len() > COMMS_HEADER_LEN && bytes[COMMS_HEADER_LEN] == 0x00 {
      defmt::warn!("Found extra 0x00 byte at position {}, skipping it", COMMS_HEADER_LEN);
    } else {
      defmt::warn!("Frame length mismatch: got {}, expected {}", bytes.len(), total);
      return None;
    }
  }

  let mut payload: CommsPayload = Vec::new();
  let copy = core::cmp::min(len, COMMS_MAX_PAYLOAD);

  // Skip extra 0x00 byte if present (workaround for HDLC deframing issue)
  let payload_start = if bytes.len() == total + 1 && bytes.len() > COMMS_HEADER_LEN && bytes[COMMS_HEADER_LEN] == 0x00 {
    COMMS_HEADER_LEN + 1 // Skip the extra byte
  } else {
    COMMS_HEADER_LEN // Normal case
  };

  if bytes.len() >= payload_start + copy {
    payload.extend_from_slice(&bytes[payload_start..payload_start + copy]).ok()?;
  } else {
    defmt::warn!("Not enough bytes for payload: need {}, have {}", payload_start + copy, bytes.len());
    return None;
  }

  Some(Message {
    command: cmd,
    id,
    fragments: frags,
    fragment: frag,
    length: len as u16,
    payload,
  })
}

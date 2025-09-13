//! HDLC over serial: minimal API with read()/write() using CommsMsg.

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;

use crate::hardware::serial;
use crate::protocol::hdlc;

// Byte vector aliases used throughout this module
pub type ByteVec = Vec<u8, 128>;
pub type FramedBuf = Vec<u8, 128>;
pub type CommsPayload = Vec<u8, COMMS_MAX_PAYLOAD>;
pub type CommsFrameBuf = Vec<u8, { COMMS_HEADER_LEN + COMMS_MAX_PAYLOAD }>; // COMMS_HEADER_LEN=11 now

// Comms message format (little-endian):
// - command:      u16
// - id:           u8
// - fragments:    u16 (total fragments)
// - fragment:     u16 (0-based index)
// - length:       u8  (payload length in bytes)
// - payload:      [u8; length]

pub const COMMS_HEADER_LEN: usize = 8;
pub const COMMS_MAX_PAYLOAD: usize = 128;

#[derive(Clone, Debug)]
pub struct CommsMsg {
  pub command: u16,
  pub id: u8,
  pub fragments: u16,
  pub fragment: u16,
  pub length: u8,
  pub payload: CommsPayload,
}

impl Default for CommsMsg {
  fn default() -> Self {
    Self {
      command: 0,
      id: 0,
      fragments: 1,
      fragment: 1,
      length: 0,
      payload: Vec::new(),
    }
  }
}

impl CommsMsg {
  /// Convenience constructor with defaults (id=0, fragments=1, fragment=1).
  pub fn new(command: u16, payload: &[u8]) -> Self {
    let mut buf: Vec<u8, COMMS_MAX_PAYLOAD> = Vec::new();
    let take = core::cmp::min(payload.len(), COMMS_MAX_PAYLOAD);
    let _ = buf.extend_from_slice(&payload[..take]);
    Self {
      command,
      id: 0,
      fragments: 1,
      fragment: 1,
      length: take as u8,
      payload: buf,
    }
  }
}

// Queue of parsed Comms messages
static COMMS_MSG_QUEUE: Channel<CriticalSectionRawMutex, CommsMsg, 8> = Channel::new();

/// Encode a CommsMsg and send over HDLC
pub fn write<W: embedded_io::Write>(serial: &mut W, msg: &CommsMsg) {
  // Build unframed message (header + payload)
  let mut buf: CommsFrameBuf = Vec::new();
  let len_usize = core::cmp::min(msg.payload.len(), COMMS_MAX_PAYLOAD);
  let len: u8 = core::cmp::min(msg.length as usize, len_usize) as u8;
  buf.extend_from_slice(&msg.command.to_le_bytes()).ok();
  buf.push(msg.id).ok();
  buf.extend_from_slice(&msg.fragments.to_le_bytes()).ok();
  buf.extend_from_slice(&msg.fragment.to_le_bytes()).ok();
  buf.push(len).ok();
  buf.extend_from_slice(&msg.payload[..len as usize]).ok();

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
pub fn read() -> Option<CommsMsg> {
  COMMS_MSG_QUEUE.try_receive().ok()
}

// --- Internal helpers ---

/// Try to decode an HDLC frame from a buffer of received serial data
fn try_decode_hdlc(buf: &mut Vec<u8, 128>, out: &mut Vec<u8, 128>) -> bool {
  hdlc::hdlc_deframe(buf, out).is_some()
}

/// Try to parse a Comms message from a byte slice (little-endian)
fn try_parse_comms_frame(bytes: &[u8]) -> Option<CommsMsg> {
  if bytes.len() < COMMS_HEADER_LEN {
    return None;
  }
  let cmd = u16::from_le_bytes([bytes[0], bytes[1]]);
  let id = bytes[2];
  let frags = u16::from_le_bytes([bytes[3], bytes[4]]);
  let frag = u16::from_le_bytes([bytes[5], bytes[6]]);
  let len = bytes[7] as usize;
  let total = COMMS_HEADER_LEN + len;
  if bytes.len() < total {
    return None;
  }
  let mut payload: CommsPayload = Vec::new();
  let copy = core::cmp::min(len, COMMS_MAX_PAYLOAD);
  payload
    .extend_from_slice(&bytes[COMMS_HEADER_LEN..COMMS_HEADER_LEN + copy])
    .ok()?;
  Some(CommsMsg {
    command: cmd,
    id,
    fragments: frags,
    fragment: frag,
    length: len as u8,
    payload,
  })
}

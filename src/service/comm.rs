//! Comms composition: HDLC framing over serial transport plus a simple message frame.

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;

use crate::protocol::hdlc;
use crate::hardware::serial;

// Queue for decoded HDLC payloads (raw bytes)
static DECODED_HDLC_QUEUE: Channel<CriticalSectionRawMutex, Vec<u8, 128>, 8> = Channel::new();

// Comms message format (little-endian):
// - command:      u16
// - id:           u32 (e.g., message id)
// - fragments:    u16 (total fragments)
// - fragment:     u16 (0-based index)
// - length:       u16 (payload length in bytes)
// - payload:      [u8; length]

pub const COMMS_HEADER_LEN: usize = 12;
pub const COMMS_MAX_PAYLOAD: usize = 128; // bounded payload for queues

#[derive(Clone, Debug)]
pub struct CommsMsg {
    pub command: u16,
    pub id: u32,
    pub fragments: u16,
    pub fragment: u16,
    pub payload: Vec<u8, COMMS_MAX_PAYLOAD>,
}

// Queue of parsed Comms messages
static COMMS_MSG_QUEUE: Channel<CriticalSectionRawMutex, CommsMsg, 8> = Channel::new();

/// Encode a Comms message (header + payload) into a byte buffer (little-endian)
pub fn encode_comms_frame(
    command: u16,
    id: u32,
    fragments: u16,
    fragment: u16,
    payload: &[u8],
) -> Vec<u8, { COMMS_HEADER_LEN + COMMS_MAX_PAYLOAD }> {
    let mut out: Vec<u8, { COMMS_HEADER_LEN + COMMS_MAX_PAYLOAD }> = Vec::new();
    let len: u16 = core::cmp::min(payload.len(), COMMS_MAX_PAYLOAD) as u16;

    // Header (LE)
    out.extend_from_slice(&command.to_le_bytes()).ok();
    out.extend_from_slice(&id.to_le_bytes()).ok();
    out.extend_from_slice(&fragments.to_le_bytes()).ok();
    out.extend_from_slice(&fragment.to_le_bytes()).ok();
    out.extend_from_slice(&len.to_le_bytes()).ok();

    // Payload
    out.extend_from_slice(&payload[..len as usize]).ok();
    out
}

/// Try to parse a Comms message from a byte slice (little-endian)
pub fn try_parse_comms_frame(bytes: &[u8]) -> Option<CommsMsg> {
    if bytes.len() < COMMS_HEADER_LEN {
        return None;
    }
    let cmd = u16::from_le_bytes([bytes[0], bytes[1]]);
    let id = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    let frags = u16::from_le_bytes([bytes[6], bytes[7]]);
    let frag = u16::from_le_bytes([bytes[8], bytes[9]]);
    let len = u16::from_le_bytes([bytes[10], bytes[11]]) as usize;
    let total = COMMS_HEADER_LEN + len;
    if bytes.len() < total {
        return None;
    }
    let mut payload: Vec<u8, COMMS_MAX_PAYLOAD> = Vec::new();
    let copy = core::cmp::min(len, COMMS_MAX_PAYLOAD);
    payload.extend_from_slice(&bytes[COMMS_HEADER_LEN..COMMS_HEADER_LEN + copy]).ok()?;
    Some(CommsMsg {
        command: cmd,
        id: id,
        fragments: frags,
        fragment: frag,
        payload,
    })
}

/// Frame and send a Comms message over HDLC
pub fn write_comms_frame<W: embedded_io::Write>(
    serial: &mut W,
    command: u16,
    id: u32,
    fragments: u16,
    fragment: u16,
    payload: &[u8],
) {
    let framed = encode_comms_frame(command, id, fragments, fragment, payload);
    write_hdlc(serial, &framed);
}

/// Encode a payload as HDLC and write to serial (blocking)
pub fn write_hdlc<W: embedded_io::Write>(serial: &mut W, payload: &[u8]) {
    let mut framed = Vec::<u8, 128>::new();
    hdlc::hdlc_frame(payload, &mut framed);
    serial::write(serial, &framed);
}

/// Try to decode an HDLC frame from a buffer of received serial data
pub fn try_decode_hdlc(buf: &mut Vec<u8, 128>, out: &mut Vec<u8, 128>) -> bool {
    hdlc::hdlc_deframe(buf, out).is_some()
}

/// Async task: read bytes from serial queue, deframe, and publish decoded payloads
#[embassy_executor::task]
pub async fn serial_hdlc_consumer_task() {
    let mut rx_buf: Vec<u8, 128> = Vec::new();
    let mut decoded: Vec<u8, 128> = Vec::new();
    loop {
        // Wait for a new message from the serial RX queue
        let msg = serial::recv_raw().await;
        // Append to buffer
        rx_buf.extend_from_slice(&msg).ok();
        // Try to decode HDLC frame(s)
        while try_decode_hdlc(&mut rx_buf, &mut decoded) {
            // Publish raw decoded payload (for compatibility)
            let mut payload = Vec::new();
            payload.extend_from_slice(&decoded).ok();
            let _ = DECODED_HDLC_QUEUE.try_send(payload);

            // Also try to parse as a Comms frame and publish
            if let Some(msg) = try_parse_comms_frame(&decoded) {
                let _ = COMMS_MSG_QUEUE.try_send(msg);
            }
        }
    }
}

/// Try to read a decoded HDLC payload (non-blocking)
pub fn try_read_decoded_hdlc() -> Option<Vec<u8, 128>> {
    DECODED_HDLC_QUEUE.try_receive().ok()
}

/// Async (blocking) read of a decoded HDLC payload
pub async fn read_decoded_hdlc() -> Vec<u8, 128> {
    DECODED_HDLC_QUEUE.receive().await
}

/// Try to read a parsed Comms message (non-blocking)
pub fn try_read_comms_msg() -> Option<CommsMsg> {
    COMMS_MSG_QUEUE.try_receive().ok()
}

/// Async (blocking) read of a parsed Comms message
pub async fn read_comms_msg() -> CommsMsg {
    COMMS_MSG_QUEUE.receive().await
}

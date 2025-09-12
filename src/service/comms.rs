//! Comms composition: HDLC framing over serial transport.

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;

use crate::protocol::hdlc;
use crate::hardware::serial;

// Queue for decoded HDLC frames
static DECODED_HDLC_QUEUE: Channel<CriticalSectionRawMutex, Vec<u8, 128>, 8> = Channel::new();

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
            let mut payload = Vec::new();
            payload.extend_from_slice(&decoded).ok();
            let _ = DECODED_HDLC_QUEUE.try_send(payload);
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

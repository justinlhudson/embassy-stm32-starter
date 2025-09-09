/// Async task: read all available bytes from UART and enqueue into SERIAL_RX_QUEUE as soon as received
#[embassy_executor::task]
pub async fn serial_rx_task(mut uart: embassy_stm32::usart::Uart<'static, Async>) {
    use heapless::String;
    loop {
        let mut buf = [0u8; 64];
        // Read as many bytes as are currently available (returns immediately if any)
        let n = match uart.read(&mut buf).await {
            Ok(n) => n,
            Err(_) => continue,
        };
        if n == 0 { continue; }
        // Convert to heapless String (truncate if needed)
        let mut s = String::<64>::new();
        let _ = s.push_str(core::str::from_utf8(&buf[..n]).unwrap_or(""));
        let _ = SERIAL_RX_QUEUE.try_send(s);
    }
}
use crate::protocol::hdlc;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::mpmc::{Channel, Receiver};
use heapless::String;

// Static RX queue for serial input (raw lines)
static SERIAL_RX_QUEUE: Channel<NoopRawMutex, String<64>, 8> = Channel::new();
// Static queue for decoded HDLC payloads
static DECODED_HDLC_QUEUE: Channel<NoopRawMutex, Vec<u8, 128>, 8> = Channel::new();
/// Encode a payload as HDLC and write to serial (blocking)
pub fn write_hdlc<'a, W: embedded_io::Write>(serial: &mut W, payload: &[u8]) {
    let mut framed = heapless::Vec::<u8, 128>::new();
    hdlc::hdlc_frame(payload, &mut framed);
    write(serial, &framed);
}

/// Try to decode an HDLC frame from a buffer of received serial data
pub fn try_decode_hdlc(buf: &mut heapless::Vec<u8, 128>, out: &mut heapless::Vec<u8, 128>) -> bool {
    hdlc::hdlc_deframe(buf, out).is_some()
}
/// Blocking write function for serial output
pub fn write<'a, W: embedded_io::Write>(serial: &mut W, data: &[u8]) {
    use embedded_io::Write as _;
    let _ = serial.write_all(data);
    let _ = serial.flush();
}

/// Try to read a string from the serial RX queue (non-blocking)
pub fn read() -> Option<String<64>> {
    let receiver = SERIAL_RX_QUEUE.receiver();
    receiver.try_recv().ok()
}

/// Example async task: read HDLC frames from serial, queue decoded payloads
#[embassy_executor::task]
pub async fn serial_hdlc_consumer_task() {
    use heapless::Vec;
    let mut rx_buf: Vec<u8, 128> = Vec::new();
    let mut decoded: Vec<u8, 128> = Vec::new();
    loop {
        // Wait for a new message from the serial RX queue
        let msg = read_async().await;
        // Append to buffer
        rx_buf.extend_from_slice(msg.as_bytes()).ok();
        // Try to decode HDLC frame(s)
        while try_decode_hdlc(&mut rx_buf, &mut decoded) {
            // decoded now contains the deframed payload
            // Queue the decoded payload for later consumption
            let mut payload = Vec::new();
            payload.extend_from_slice(&decoded).ok();
            let _ = DECODED_HDLC_QUEUE.try_send(payload);
        }
    }
}

/// Try to read a decoded HDLC payload (non-blocking)
pub fn read_decoded_hdlc() -> Option<Vec<u8, 128>> {
    let receiver = DECODED_HDLC_QUEUE.receiver();
    receiver.try_recv().ok()
}

/// Async (blocking) read of a decoded HDLC payload
pub async fn read_decoded_hdlc_async() -> Vec<u8, 128> {
    let receiver = DECODED_HDLC_QUEUE.receiver();
    receiver.recv().await
}
// Serial (USART2) abstraction for Nucleo-F446RE


use embassy_stm32::usart::{Uart, Config as UartConfig};
use embassy_stm32::{peripherals, usart};
use embassy_stm32::mode::Async;
use embassy_stm32::bind_interrupts;

// Bind USART2 interrupt for async mode


// SAFETY: This macro is required by Embassy for async UART operation. Must be at module root.
embassy_stm32::bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

pub fn init<'a>(
    usart2: peripherals::USART2,
    tx: peripherals::PA2,
    rx: peripherals::PA3,
) -> Uart<'a, Async> {
    let mut config = UartConfig::default();
    config.baudrate = 115_200;
    Uart::new(usart2, tx, rx, (), (), Irqs, config).unwrap()
}

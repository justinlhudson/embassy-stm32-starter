use crate::protocol::hdlc;
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

/// Read a string from the serial RX queue (blocking)
pub fn read() -> Option<heapless::String<64>> {
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    let receiver = SERIAL_RX_QUEUE.receiver();
    receiver.try_recv().ok()
}

/// Blocking read from the serial RX queue (waits for a message)
pub async fn read_async() -> heapless::String<64> {
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    let receiver = SERIAL_RX_QUEUE.receiver();
    receiver.recv().await
}

/// Example async task: read HDLC frames from serial and process them
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
            // Process decoded as needed (e.g., print, parse, etc.)
        }
    }
}
//! Serial (USART2) abstraction for Nucleo-F446RE


use embassy_stm32::usart::{Uart, Config as UartConfig};
use embassy_stm32::{peripherals, usart};
use embassy_stm32::mode::Async;
use embassy_stm32::bind_interrupts;

// Bind USART2 interrupt for async mode
bind_interrupts!(struct Irqs {
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

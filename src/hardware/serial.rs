/// Async task: read all available bytes from UART and enqueue into SERIAL_RX_QUEUE as soon as received  
#[embassy_executor::task]
pub async fn serial_rx_task() {
    // TODO: Implement UART reading when embassy API is fixed
    use heapless::String;
    loop {
        // Placeholder - just add test data to queue
        let mut s = String::<64>::new();
        let _ = s.push_str("test");
        let _ = SERIAL_RX_QUEUE.try_send(s);
        Timer::after(Duration::from_millis(1000)).await;
    }
}
use crate::protocol::hdlc;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Timer, Duration};
use heapless::{String, Vec};

// Global queues for serial communication
static SERIAL_RX_QUEUE: Channel<CriticalSectionRawMutex, String<64>, 8> = Channel::new();
// Queue for decoded HDLC frames  
static DECODED_HDLC_QUEUE: Channel<CriticalSectionRawMutex, Vec<u8, 128>, 8> = Channel::new();
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

    let _ = serial.write_all(data);
    let _ = serial.flush();
}

/// Try to read a string from the serial RX queue (non-blocking)
pub fn read() -> Option<String<64>> {
    SERIAL_RX_QUEUE.try_receive().ok()
}

/// Example async task: read HDLC frames from serial, queue decoded payloads
#[embassy_executor::task]
pub async fn serial_hdlc_consumer_task() {
    use heapless::Vec;
    let mut rx_buf: Vec<u8, 128> = Vec::new();
    let mut decoded: Vec<u8, 128> = Vec::new();
    loop {
        // Wait for a new message from the serial RX queue
        let msg = SERIAL_RX_QUEUE.receive().await;
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
    DECODED_HDLC_QUEUE.try_receive().ok()
}

/// Async (blocking) read of a decoded HDLC payload
pub async fn read_decoded_hdlc_async() -> Vec<u8, 128> {
    DECODED_HDLC_QUEUE.receive().await
}
// Serial (USART2) abstraction for Nucleo-F446RE
// TODO: Implement proper UART initialization


// Bind USART2 interrupt for async mode


// SAFETY: This macro is required by Embassy for async UART operation. Must be at module root.
// TODO: Fix UART initialization for Embassy v0.4.0
// embassy_stm32::bind_interrupts!(struct Irqs {
//     USART2 => usart::InterruptHandler<peripherals::USART2>;
// });

// pub fn init(
//     usart2: peripherals::USART2,
//     tx: peripherals::PA2, 
//     rx: peripherals::PA3,
// ) -> Uart<'static, Async> {
//     let mut config = UartConfig::default();
//     config.baudrate = 115_200;
//     // Embassy expects: new(peri, rx_pin, tx_pin, cts_pin, rts_pin, irqs, config)
//     Uart::new(usart2, rx, tx, embassy_stm32::gpio::NoPin, embassy_stm32::gpio::NoPin, Irqs, config)
// }

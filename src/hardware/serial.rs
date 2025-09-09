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
use embassy_sync::channel::mpmc::{Channel, Sender, Receiver};
use heapless::consts::*;
use heapless::Vec;
use core::cell::RefCell;
use core::future::Future;
// Static queue for received serial data
pub static SERIAL_RX_QUEUE: Channel<heapless::String<64>, 4> = Channel::new();
/// Async task to consume serial reads and store them in a queue
#[embassy_executor::task]
pub async fn serial_read_task<R: embedded_io_async::Read + 'static>(mut serial: R) {
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    let sender = SERIAL_RX_QUEUE.sender();
    let mut buf = [0u8; 64];
    loop {
        match serial.read(&mut buf).await {
            Ok(n) if n > 0 => {
                if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                    let mut msg = heapless::String::<64>::new();
                    let _ = msg.push_str(s);
                    let _ = sender.try_send(msg);
                }
            }
            _ => {}
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

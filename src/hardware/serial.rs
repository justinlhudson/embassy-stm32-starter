/*!
# Serial Communication Module with DMA and Idle Interrupt

This module provides DMA-based serial communication with idle interrupt detection
for efficient data reception on STM32F446RE using Embassy framework.

## Key Features

- **DMA-based reception**: Uses Embassy's built-in DMA support for efficient data transfer
- **Idle interrupt detection**: Automatically detects when transmission ends using UART idle interrupt
- **Async/await support**: Fully async using Embassy's executor
- **HDLC protocol support**: Integrated with HDLC framing protocol

## Usage

1. Create a UART instance in your main application
2. Split it into TX/RX parts  
3. Pass the RX part to `create_serial_receiver`
4. Spawn the `serial_rx_task_dma` task
5. Use `read()` and `read_decoded_hdlc()` functions to get data

## Example

```rust
// In your main function after embassy_stm32::init():
let uart = Uart::new(p.USART2, p.PA3, p.PA2, p.DMA1_CH6, p.DMA1_CH5, Irqs, config)?;
let (uart_tx, uart_rx) = uart.split();
let serial_rx = create_serial_receiver(uart_rx);
spawner.spawn(serial_rx_task_dma(serial_rx))?;
```
*/

use crate::protocol::hdlc;
use embassy_executor::Spawner;
use embassy_stm32::{
    usart::{self, Uart, UartRx, UartTx, Config as UartConfig, Instance, RxPin, TxPin, TxDma, RxDma},
    mode::Async,
    bind_interrupts,
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Timer, Duration};
use heapless::{String, Vec};

// Bind USART2 interrupt handler for async operation  
bind_interrupts!(pub struct Irqs {
    USART2 => usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
});

// Also expose a binding for USART3 for boards that use it (e.g., Nucleo-144 F413ZH)
bind_interrupts!(pub struct IrqsUsart3 {
    USART3 => usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
});

// DMA-based serial receiver with idle interrupt detection
pub struct SerialReceiver {
    uart_rx: UartRx<'static, Async>,
    rx_buffer: [u8; 256],
    buffer_pos: usize,
}

impl SerialReceiver {
    pub fn new(uart_rx: UartRx<'static, Async>) -> Self {
        Self {
            uart_rx,
            rx_buffer: [0; 256],
            buffer_pos: 0,
        }
    }

    /// Read with idle detection - returns data when idle interrupt occurs
    /// This uses Embassy's built-in DMA with idle interrupt functionality
    pub async fn read_until_idle(&mut self) -> Result<&[u8], embassy_stm32::usart::Error> {
        // Use read_until_idle method from Embassy UART
        // This automatically handles DMA transfer with idle interrupt
        match self.uart_rx.read_until_idle(&mut self.rx_buffer).await {
            Ok(len) => {
                self.buffer_pos = len;
                Ok(&self.rx_buffer[..len])
            }
            Err(e) => Err(e)
        }
    }

    /// Get current buffer contents
    pub fn get_buffer(&self) -> &[u8] {
        &self.rx_buffer[..self.buffer_pos]
    }

    /// Clear buffer
    pub fn clear_buffer(&mut self) {
        self.buffer_pos = 0;
    }
}

/// Create a SerialReceiver from a UartRx
/// This should be called after you've created a UART instance and split it
pub fn create_serial_receiver(uart_rx: UartRx<'static, Async>) -> SerialReceiver {
    SerialReceiver::new(uart_rx)
}

/// Async task: read from UART using DMA with idle interrupt
/// This task uses Embassy's built-in DMA and idle interrupt functionality
#[embassy_executor::task]
pub async fn serial_rx_task_dma(mut serial_rx: SerialReceiver) {
    loop {
        match serial_rx.read_until_idle().await {
            Ok(data) => {
                if !data.is_empty() {
                    // Convert to string and queue
                    if let Ok(s) = core::str::from_utf8(data) {
                        let mut string: String<64> = String::new();
                        if string.push_str(s).is_ok() {
                            let _ = SERIAL_RX_QUEUE.try_send(string);
                        }
                    }
                }
                serial_rx.clear_buffer();
            }
            Err(_e) => {
                // Handle error - could log with defmt if needed
                // For now, just wait a bit and try again
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}

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

/// Get the interrupt handler type for USART2 (for export to board configs)
pub type SerialIrqs = Irqs;
pub type Serial3Irqs = IrqsUsart3;

/// Example initialization function - this shows how to use the serial module
/// You would call this from your main application after embassy_stm32::init()
/// 
/// ```rust,no_run
/// # use embassy_stm32::{Config as UartConfig, usart::Uart};
/// # let p = embassy_stm32::init(Default::default());
/// let mut uart_config = UartConfig::default();
/// uart_config.baudrate = 115_200;
/// 
/// let uart = Uart::new(
///     p.USART2, 
///     p.PA3,    // RX pin
///     p.PA2,    // TX pin  
///     p.DMA1_CH6, // RX DMA channel
///     p.DMA1_CH5, // TX DMA channel
///     SerialIrqs, 
///     uart_config
/// ).unwrap();
/// 
/// let (uart_tx, uart_rx) = uart.split();
/// let serial_receiver = create_serial_receiver(uart_rx);
/// 
/// // Spawn the DMA receive task
/// spawner.spawn(serial_rx_task_dma(serial_receiver)).unwrap();
/// 
/// // Spawn the HDLC decoder task
/// spawner.spawn(serial_hdlc_consumer_task()).unwrap();
/// ```
pub fn _example_usage_documentation_only() {
    // This function exists only for documentation purposes
    // It will be optimized away in release builds
    unimplemented!("This is documentation only")
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

/// Board-agnostic helper to initialize USART2 with DMA and spawn serial tasks.
/// For Nucleo-F446RE: PA2=TX, PA3=RX; TX DMA=DMA1_CH6, RX DMA=DMA1_CH5.
/// Returns the TX half for writes.
pub fn init_usart2(spawner: Spawner, p: embassy_stm32::Peripherals) -> UartTx<'static, Async> {
    let mut cfg = UartConfig::default();
    cfg.baudrate = 115_200;

    // Order: peri, rx, tx, Irqs, tx_dma, rx_dma, config
    let uart = Uart::new(
        p.USART2,
        p.PA3,      // RX
        p.PA2,      // TX
        Irqs,
        p.DMA1_CH6, // TX DMA
        p.DMA1_CH5, // RX DMA
        cfg,
    )
    .unwrap();

    let (tx, rx) = uart.split();
    let receiver = create_serial_receiver(rx);
    let _ = spawner.spawn(serial_rx_task_dma(receiver));
    let _ = spawner.spawn(serial_hdlc_consumer_task());
    tx
}

/// Generic serial initializer: takes USART peri, RX/TX pins, Irqs binding, TX/RX DMA, sets 115200 and spawns tasks.
pub fn init_serial<T, RX, TX, TXDMA, RXDMA>(
    spawner: Spawner,
    usart: Peri<'static, T>,
    rx: Peri<'static, RX>,
    tx: Peri<'static, TX>,
    irqs: impl embassy_stm32::interrupt::typelevel::Binding<
        <T as Instance>::Interrupt,
        usart::InterruptHandler<T>,
    > + 'static,
    tx_dma: Peri<'static, TXDMA>,
    rx_dma: Peri<'static, RXDMA>,
) -> UartTx<'static, Async>
where
    T: Instance + 'static,
    RX: RxPin<T> + 'static,
    TX: TxPin<T> + 'static,
    TXDMA: TxDma<T> + 'static,
    RXDMA: RxDma<T> + 'static,
{
    let mut cfg = UartConfig::default();
    cfg.baudrate = 115_200;

    let uart = Uart::new(usart, rx, tx, irqs, tx_dma, rx_dma, cfg).unwrap();
    let (tx, rx) = uart.split();
    let receiver = create_serial_receiver(rx);
    let _ = spawner.spawn(serial_rx_task_dma(receiver));
    let _ = spawner.spawn(serial_hdlc_consumer_task());
    tx
}

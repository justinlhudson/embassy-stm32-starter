use embassy_executor::Spawner;
use embassy_stm32::{
  Peri, bind_interrupts,
  mode::Async,
  usart::{self, Config as UartConfig, Instance, RxDma, RxPin, TxDma, TxPin, Uart, UartRx, UartTx},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use heapless::Vec;

// Define a constant for buffer size
const SERIAL_BUFFER_SIZE: usize = 256;
const SERIAL_QUEUE_DEPTH: usize = 4;
const SERIAL_BAUDRATE: u32 = 115_200;

// Bind USART2 interrupt handler for async operation
bind_interrupts!(pub struct Irqs {
    USART2 => usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
});

// Also expose a binding for USART3 for boards that use it (e.g., Nucleo-144 F413ZH)
bind_interrupts!(pub struct IrqsUsart3 {
    USART3 => usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
});

// Also expose a binding for USART6 for boards that use it (e.g., Nucleo-144 F413ZH VCP)
bind_interrupts!(pub struct IrqsUsart6 {
    USART6 => usart::InterruptHandler<embassy_stm32::peripherals::USART6>;
});

// DMA-based serial receiver with idle interrupt detection
pub struct SerialReceiver<'a> {
  uart_rx: UartRx<'static, Async>,
  rx_buffer: &'a Mutex<CriticalSectionRawMutex, [u8; SERIAL_BUFFER_SIZE]>,
  buffer_pos: usize,
}

impl<'a> SerialReceiver<'a> {
  pub fn new(uart_rx: UartRx<'static, Async>, rx_buffer: &'a Mutex<CriticalSectionRawMutex, [u8; SERIAL_BUFFER_SIZE]>) -> Self {
    Self {
      uart_rx,
      rx_buffer,
      buffer_pos: 0,
    }
  }

  /// Read with idle detection - returns data when idle interrupt occurs
  /// This uses Embassy's built-in DMA with idle interrupt functionality
  pub async fn read_until_idle(&mut self) -> Result<Vec<u8, SERIAL_BUFFER_SIZE>, embassy_stm32::usart::Error> {
    let mut buffer = self.rx_buffer.lock().await;
    match self.uart_rx.read_until_idle(&mut *buffer).await {
      Ok(len) => {
        self.buffer_pos = len;
        let mut result = Vec::new();
        result.extend_from_slice(&buffer[..len]).ok();
        Ok(result)
      }
      Err(e) => Err(e),
    }
  }

  /// Get current buffer contents
  pub async fn get_buffer(&self) -> Vec<u8, SERIAL_BUFFER_SIZE> {
    let buffer = self.rx_buffer.lock().await;
    let mut result = Vec::new();
    result.extend_from_slice(&buffer[..self.buffer_pos]).ok();
    result
  }

  /// Clear buffer
  pub async fn clear_buffer(&mut self) {
    let mut buffer = self.rx_buffer.lock().await;
    buffer.fill(0);
    self.buffer_pos = 0;
  }
}

/// Create a SerialReceiver from a UartRx
/// This should be called after you've created a UART instance and split it
pub fn create_serial_receiver(uart_rx: UartRx<'static, Async>) -> SerialReceiver<'static> {
  SerialReceiver::new(uart_rx, &SHARED_RX_BUFFER)
}

/// Async task: read from UART using DMA with idle interrupt
/// This task uses Embassy's built-in DMA and idle interrupt functionality
#[embassy_executor::task]
pub async fn serial_rx_task_dma(mut serial_rx: SerialReceiver<'static>) {
  loop {
    match serial_rx.read_until_idle().await {
      Ok(data) => {
        if !data.is_empty() {
          // Copy bytes into a bounded buffer and queue
          let mut bytes: Vec<u8, SERIAL_BUFFER_SIZE> = Vec::new();
          let take = core::cmp::min(bytes.capacity(), data.len());
          bytes.extend_from_slice(&data[..take]).ok();
          let _ = SERIAL_RX_QUEUE.try_send(bytes);
        }
        serial_rx.clear_buffer().await;
      }
      Err(_e) => {
        // Handle error - could log with defmt if needed
        // For now, just wait a bit and try again
        Timer::after(Duration::from_millis(10)).await;
      }
    }
  }
}

// Global queue for raw serial bytes
static SERIAL_RX_QUEUE: Channel<CriticalSectionRawMutex, Vec<u8, SERIAL_BUFFER_SIZE>, SERIAL_QUEUE_DEPTH> = Channel::new();
/// Blocking write function for serial output
pub fn write<W: embedded_io::Write>(serial: &mut W, data: &[u8]) {
  let _ = serial.write_all(data);
  let _ = serial.flush();
}

/// Try to read raw serial bytes (non-blocking)
pub fn read() -> Option<Vec<u8, SERIAL_BUFFER_SIZE>> {
  SERIAL_RX_QUEUE.try_receive().ok()
}

/// Await raw serial bytes from the RX queue
pub async fn recv_raw() -> Vec<u8, SERIAL_BUFFER_SIZE> {
  SERIAL_RX_QUEUE.receive().await
}

/// Get the interrupt handler type aliases for export to board configs
pub use Irqs as Serial2Irqs;
pub use IrqsUsart3 as Serial3Irqs;
pub use IrqsUsart6 as Serial6Irqs;

/// Generic serial initializer: takes USART peri, RX/TX pins, Irqs binding, TX/RX DMA, sets 115200 and spawns tasks.
pub fn init_serial<T, RX, TX, TXDMA, RXDMA>(
  spawner: Spawner,
  usart: Peri<'static, T>,
  rx: Peri<'static, RX>,
  tx: Peri<'static, TX>,
  irqs: impl embassy_stm32::interrupt::typelevel::Binding<<T as Instance>::Interrupt, usart::InterruptHandler<T>> + 'static,
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
  cfg.baudrate = SERIAL_BAUDRATE;

  let uart = Uart::new(usart, rx, tx, irqs, tx_dma, rx_dma, cfg).unwrap();
  let (tx, rx) = uart.split();
  let receiver = create_serial_receiver(rx);
  let _ = spawner.spawn(serial_rx_task_dma(receiver));
  let _ = spawner.spawn(crate::service::comm::serial_hdlc_consumer_task());
  tx
}

// Define a shared buffer to reduce RAM usage
static SHARED_RX_BUFFER: Mutex<CriticalSectionRawMutex, [u8; SERIAL_BUFFER_SIZE]> = Mutex::new([0; SERIAL_BUFFER_SIZE]);

// Shared RX buffer for SerialReceiver (to avoid stack allocation in async task)
use crate::hardware::timers::TimingUtils;
use embassy_sync::mutex::Mutex;
// Shared buffer for serial RX (mirrors comm.rs approach)
static SERIAL_SHARED_BUF: Mutex<CriticalSectionRawMutex, Vec<u8, SERIAL_BUFFER_SIZE>> = Mutex::new(Vec::new());
use embassy_executor::Spawner;
use embassy_stm32::{
  Peri, bind_interrupts,
  mode::Async,
  usart::{self, Config as UartConfig, Instance, RxDma, RxPin, TxDma, TxPin, Uart, UartRx, UartTx},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
// ...existing code...
use heapless::Vec;

/// Serial RX buffer size (bytes)
pub const SERIAL_BUFFER_SIZE: usize = 1024;
pub const SERIAL_QUEUE_DEPTH: usize = 4;

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

// Shared RX buffer for SerialReceiver (to avoid stack allocation in async task)
static SERIAL_RX_BUF: Mutex<CriticalSectionRawMutex, [u8; SERIAL_BUFFER_SIZE]> = Mutex::new([0; SERIAL_BUFFER_SIZE]);

// DMA-based serial receiver with idle interrupt detection
pub struct SerialReceiver {
  uart_rx: UartRx<'static, Async>,
  buffer_pos: usize,
}

impl SerialReceiver {
  pub fn new(uart_rx: UartRx<'static, Async>) -> Self {
    Self { uart_rx, buffer_pos: 0 }
  }
  // Shared RX buffer for SerialReceiver (to avoid stack allocation in async task)

  /// Read with idle detection - returns data when idle interrupt occurs
  /// This uses Embassy's built-in DMA with idle interrupt functionality
  pub async fn read_until_idle(&mut self) -> Result<heapless::Vec<u8, SERIAL_BUFFER_SIZE>, embassy_stm32::usart::Error> {
    // Use a static RX buffer protected by a mutex
    let mut buf_guard = SERIAL_RX_BUF.lock().await;
    let rx_buffer = &mut *buf_guard;
    match self.uart_rx.read_until_idle(rx_buffer).await {
      Ok(len) => {
        self.buffer_pos = len;
        // Copy data out to an owned Vec
        let mut out = heapless::Vec::<u8, SERIAL_BUFFER_SIZE>::new();
        let _ = out.extend_from_slice(&rx_buffer[..len]);
        Ok(out)
      }
      Err(e) => Err(e),
    }
  }

  /// Get current buffer contents
  pub fn get_buffer(&self) -> &[u8] {
    // Not used in async context; returns empty slice
    &[]
  }

  /// Clear buffer
  pub fn clear_buffer(&mut self) {
    self.buffer_pos = 0;
    // Optionally clear static buffer, but not strictly needed
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
          // Use the shared buffer for RX
          let mut buf_guard = SERIAL_SHARED_BUF.lock().await;
          let buf = &mut *buf_guard;
          buf.clear();
          let take = core::cmp::min(buf.capacity(), data.len());
          buf.extend_from_slice(&data[..take]).ok();
          let _ = SERIAL_RX_QUEUE.try_send(buf.clone());
        }
        serial_rx.clear_buffer();
      }
      Err(_e) => {
        TimingUtils::delay_ms(10).await;
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
/// Returns a clone of the shared buffer contents (valid until next lock)
pub fn read() -> Option<Vec<u8, SERIAL_BUFFER_SIZE>> {
  SERIAL_RX_QUEUE.try_receive().ok()
}
/// Returns a clone of the shared buffer contents (valid until next lock)
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
  cfg.baudrate = 115_200;

  let uart = match Uart::new(usart, rx, tx, irqs, tx_dma, rx_dma, cfg) {
    Ok(u) => u,
    Err(e) => {
      defmt::error!("UART init failed: {:?}", e);
      panic!("UART init failed");
    }
  };
  let (tx, rx) = uart.split();
  let receiver = create_serial_receiver(rx);
  let _ = spawner.spawn(serial_rx_task_dma(receiver));
  let _ = spawner.spawn(crate::service::comm::serial_hdlc_consumer_task());
  tx
}

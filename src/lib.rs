#![no_main]
#![no_std]

// ---- compile-time MCU feature validation ---------------------------------
#[cfg(all(feature = "stm32f413", feature = "stm32f446"))]
compile_error!("Enable only one MCU feature (`stm32f413` OR `stm32f446`).");

#[cfg(not(any(feature = "stm32f413", feature = "stm32f446")))]
compile_error!(
  "No MCU feature selected. Build with `--features stm32f413` or `--features stm32f446`."
);

// ---- runtime / panic / logging plumbing ---------------------------------
use cortex_m as _; // core peripherals
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use embassy_stm32 as _; // pulls in interrupt vectors

pub use defmt::*;
pub use embassy_time::Timer;

// ---- module tree ---------------------------------------------------------
pub mod hardware {
  pub mod flash;
  pub mod gpio;
  pub mod hardfault;
  pub mod serial;
  pub mod timers;
  pub use gpio::GpioDefaults;
  pub use serial::*;
  pub use timers::*;
}

pub mod protocol {
  pub mod hdlc;
}

pub mod service {
  pub mod comm;
}

pub mod common {
  pub mod tasks;
}

pub mod board;

/// Convenience re-exports for binaries.
pub mod prelude {
  pub use crate::board::{Board, BoardConfig, BoardHardware};
  pub use crate::hardware::Timing;
  pub use crate::service::comm;
  pub use defmt::*;
  pub use embassy_time::Timer;
  pub use embedded_io::Write as _;
}

#![no_main]
#![no_std]

use cortex_m as _; // import to get the core peripherals
use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use embassy_stm32 as _; // import to get the interrupt vectors

pub use defmt::*; // re-export all defmt macros for convenience

// Hardware abstraction layer modules
pub mod hardware {
  pub mod gpio;
  pub mod serial;
  pub mod timers;
  pub use gpio::*;
  pub use serial::*;
  pub use timers::*;
}

// Services layer
pub mod service {
  pub mod comm;
  pub use comm::*;
}

// Protocol modules
pub mod protocol {
  pub mod hdlc;
  pub use hdlc::*;
}

// Common/shared functionality modules
pub mod common {
  pub mod tasks;
  pub use tasks::*;
}

// Convenience prelude for commonly used traits/types in binaries
pub mod prelude {
  pub use embedded_io::Write as _;
}

// Board configuration - included from root board.rs file (copied by setup.sh)
#[path = "../board.rs"]
pub mod board;

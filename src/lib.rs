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
    pub mod timers;
    pub use gpio::*;
    pub use timers::*;
}

// Common/shared functionality modules  
pub mod common {
    pub mod tasks;
    pub use tasks::*;
}

// Board configuration - included from root board.rs file (copied by setup.sh)
#[path = "../board.rs"]
pub mod board;

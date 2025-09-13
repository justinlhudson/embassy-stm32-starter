#![no_std]
#![no_main]

use embassy_stm32_starter::*;
use semihosting::process; // import from lib.rs

#[cortex_m_rt::entry]
fn main() -> ! {
  let _p = embassy_stm32::init(Default::default());

  // Basic integration test - if this compiles and links successfully,
  // it means the project structure and dependencies are correct
  info!("Integration test started");

  cortex_m::asm::delay(100);

  info!("Integration test completed successfully");

  process::exit(0)
}

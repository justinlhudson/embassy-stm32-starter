#![no_std]
#![no_main]

//! Flash HIL test. By default, only the non-destructive read path is
//! exercised. Enable the `flash-destructive` feature to also run the erase
//! + write + verify round-trip. **Doing so will erase the storage region.**

use cortex_m_rt::entry;
use defmt::info;
use embassy_stm32_starter::hardware::flash;
use semihosting::process;

#[entry]
fn main() -> ! {
  let p = embassy_stm32::init(Default::default());
  flash::init(p.FLASH);

  info!("Flash test starting...");

  let start = flash::start();
  let end = flash::end();
  let size = flash::size();
  info!("Flash storage region:");
  info!("  Start: 0x{:08X}", start);
  info!("  End:   0x{:08X}", end);
  info!("  Size:  {} bytes ({} KB)", size, size / 1024);

  // Always-safe read.
  let mut buf = [0u8; 16];
  match flash::read_block(0, &mut buf) {
    Ok(()) => info!("Flash read OK: {:02X}", buf),
    Err(_) => {
      info!("Flash read FAILED");
      process::exit(1);
    }
  }

  #[cfg(feature = "flash-destructive")]
  {
    info!("Destructive flash test: erase + write + verify");
    flash::erase().unwrap();

    let data: [u8; 16] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00];
    flash::write_block(flash::start(), &data).unwrap();

    let mut verify = [0u8; 16];
    flash::read_block(0, &mut verify).unwrap();
    if verify == data {
      info!("Flash round-trip OK");
    } else {
      info!("Flash round-trip FAIL: expected {:02X}, got {:02X}", data, verify);
      process::exit(1);
    }
  }

  #[cfg(not(feature = "flash-destructive"))]
  info!("(skipping destructive round-trip; enable `flash-destructive`)");

  process::exit(0)
}

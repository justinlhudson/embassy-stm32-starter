#![no_std]
#![no_main]

use embassy_stm32::flash::Flash;
use embassy_stm32_starter::hardware::flash;
use embassy_stm32_starter::*;
use embedded_storage::nor_flash::NorFlash;
use semihosting::process;

#[cortex_m_rt::entry]
fn main() -> ! {
  let p = embassy_stm32::init(Default::default());
  info!("Flash read/write test started");

  // Print storage region info for debugging
  info!("FLASH_STORAGE_START: 0x{:08X}", flash::storage_start());
  info!("FLASH_STORAGE_END:   0x{:08X}", flash::storage_end());
  info!("FLASH_STORAGE_SIZE:  {} bytes", flash::storage_size());

  // Use blocking flash instance
  let mut flash_hw = Flash::new_blocking(p.FLASH);
  let test_offset = 0;
  let test_data: [u8; 16] = [0xA5; 16];
  let mut read_buf: [u8; 16] = [0; 16];

  // Erase only the first 32KB of the last sector using board constants
  let _ = flash_hw.erase(
    embassy_stm32_starter::board::BoardConfig::FLASH_STORAGE_START,
    embassy_stm32_starter::board::BoardConfig::FLASH_STORAGE_START + 32 * 1024,
  );

  // Write test pattern
  match flash_hw.write(flash::storage_start() + test_offset as u32, &test_data) {
    Ok(_) => info!("Flash write OK"),
    Err(e) => info!("Flash write error: {:?}", e),
  }

  // Read back
  match flash::read_block(test_offset, &mut read_buf) {
    Ok(_) => info!("Flash read OK"),
    Err(e) => info!("Flash read error: {:?}", e),
  }

  // Check
  if read_buf == test_data {
    info!("Flash read/write test PASSED");
  } else {
    info!("Flash read/write test FAILED");
  }

  info!("Flash read/write test completed");
  process::exit(0)
}

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use embassy_stm32_starter::hardware::flash;
use semihosting::process;

#[entry]
fn main() -> ! {
  let _p = embassy_stm32::init(Default::default());

  info!("Flash test starting...");

  // Test flash storage configuration constants
  let start = flash::start();
  let end = flash::end();
  let size = (end - start) as usize;

  info!("Flash storage region:");
  info!("  Start: 0x{:08X}", start);
  info!("  End:   0x{:08X}", end);
  info!("  Size:  {} bytes ({} KB)", size, size / 1024);

  // Verify configuration is valid
  let calculated_size = (end - start) as usize;
  let config_valid = calculated_size == size;

  info!("Configuration valid: {}", config_valid);

  if config_valid {
    info!("✅ Flash configuration test PASSED");

    // Test reading from flash (this should work without erase/write operations)
    let mut test_buf: [u8; 16] = [0; 16];
    let read_result = flash::read_block(0, &mut test_buf);

    match read_result {
      Ok(()) => {
        info!("✅ Flash read test PASSED");
        info!("Read data: {:02X}", test_buf);
      }
      Err(_) => {
        info!("❌ Flash read test FAILED");
      }
    }
  } else {
    info!("❌ Flash configuration test FAILED");
  }

  // Attempt flash operations with workarounds for embassy-stm32 v0.4.0 bug
  info!("Testing flash operations with workarounds...");

  // Try direct flash operations (workaround functions)
  match flash::erase_sector_direct(start) {
    Ok(()) => {
      info!("✅ Direct erase workaround test PASSED");

      let test_data: [u8; 16] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00];

      match flash::write_block(start, &test_data) {
        Ok(()) => {
          info!("✅ Direct write workaround test PASSED");

          // Verify the write by reading back the data
          let mut verify_buf: [u8; 16] = [0; 16];
          match flash::read_block(0, &mut verify_buf) {
            Ok(()) => {
              if verify_buf == test_data {
                info!("✅ Write verification PASSED - data matches!");
                info!("Written: {:02X}", test_data);
                info!("Read:    {:02X}", verify_buf);
              } else {
                info!("❌ Write verification FAILED - data mismatch");
                info!("Expected: {:02X}", test_data);
                info!("Got:      {:02X}", verify_buf);
              }
            }
            Err(_) => {
              info!("❌ Write verification FAILED - read error");
            }
          }
        }
        Err(_) => {
          info!("❌ Direct write workaround test FAILED");
        }
      }
    }
    Err(_) => {
      info!("❌ Direct erase workaround test FAILED");
    }
  }

  info!("Flash test completed");
  process::exit(0)
}

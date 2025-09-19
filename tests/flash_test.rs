#![no_std]
#![no_main]

use embassy_stm32_starter::hardware::flash;
use embassy_stm32::flash::Flash;
use embassy_stm32_starter::*;
use semihosting::process;

#[cortex_m_rt::entry]
fn main() -> ! {
    let _p = embassy_stm32::init(Default::default());
    info!("Flash read/write test started");

    let mut flash_hw = unsafe { Flash::new_blocking(_p.FLASH) };
    let test_offset = 0;
    let test_data: [u8; 16] = [0xA5; 16];
    let mut read_buf: [u8; 16] = [0; 16];

    // Erase before writing
    let _ = flash::erase_blocks(&mut flash_hw);

    // Write test pattern
    match flash::write_block(&mut flash_hw, test_offset, &test_data) {
        Ok(_) => info!("Flash write OK"),
        Err(e) => info!("Flash write error: {:?}", e),
    }

    // Read back
    match flash::read_block(&mut flash_hw, test_offset, &mut read_buf) {
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

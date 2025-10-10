// Simple flash storage for STM32 using last sector
/// Provides block read/write APIs for persistent storage
use embassy_stm32::bind_interrupts;
bind_interrupts!(pub struct IrqsFlash {
  FLASH => embassy_stm32::flash::InterruptHandler;
});

use crate::board::BoardConfig;
use core::ptr;
use embassy_stm32::flash::{Error, Flash};

// Direct flash operations using register addresses (STM32F4 reference manual)
// Flash register base addresses for STM32F4xx series
const FLASH_BASE: u32 = 0x40023C00;
const FLASH_KEYR: u32 = FLASH_BASE + 0x04;
const FLASH_SR: u32 = FLASH_BASE + 0x0C;
const FLASH_CR: u32 = FLASH_BASE + 0x10;

// Flash keys for unlocking
const FLASH_KEY1: u32 = 0x45670123;
const FLASH_KEY2: u32 = 0xCDEF89AB;

// Flash control register bits
const FLASH_CR_PG: u32 = 1 << 0; // Programming
const FLASH_CR_SER: u32 = 1 << 1; // Sector Erase  
const FLASH_CR_STRT: u32 = 1 << 16; // Start
const FLASH_CR_LOCK: u32 = 1 << 31; // Lock

// Flash status register bits
const FLASH_SR_BSY: u32 = 1 << 16; // Busy flag

/// The start address of the storage region (last sector)
pub fn storage_start() -> u32 {
  BoardConfig::FLASH_STORAGE_START
}
pub fn storage_size() -> usize {
  BoardConfig::FLASH_STORAGE_SIZE
}
pub fn storage_end() -> u32 {
  BoardConfig::FLASH_STORAGE_END
}

/// Write a block of data to flash storage
pub async fn write_block(flash: &mut Flash<'_>, offset: usize, data: &[u8]) -> Result<(), Error> {
  let addr = storage_start() + offset as u32;
  let end_addr = addr + data.len() as u32;
  if end_addr > storage_end() {
    defmt::error!("Attempt to write past end of storage: addr=0x{:08X}, end=0x{:08X}", addr, end_addr);
    return Err(Error::Size);
  }
  flash.write(addr, data).await
}

/// Read a block of data from flash storage
pub fn read_block(offset: usize, buf: &mut [u8]) -> Result<(), Error> {
  let addr = storage_start() + offset as u32;
  let end_addr = addr + buf.len() as u32;
  if end_addr > storage_end() {
    defmt::error!("Attempt to read past end of storage: addr=0x{:08X}, end=0x{:08X}", addr, end_addr);
    return Err(Error::Size);
  }
  let flash_ptr = addr as *const u8;
  unsafe {
    ptr::copy_nonoverlapping(flash_ptr, buf.as_mut_ptr(), buf.len());
  }
  Ok(())
}

/// Erase the storage region (entire last sector)
pub async fn erase_blocks(flash: &mut Flash<'_>) -> Result<(), embassy_stm32::flash::Error> {
  let start = storage_start();
  let end = storage_end();
  flash.erase(start, end).await
}

/// Try small page-aligned erase operation (workaround attempt)
pub async fn erase_small_block(flash: &mut Flash<'_>, offset: usize, size: usize) -> Result<(), embassy_stm32::flash::Error> {
  let start = storage_start() + offset as u32;
  let end = start + size as u32;

  // Ensure we don't go past our allocated region
  if end > storage_end() {
    defmt::error!("Erase operation would exceed storage bounds");
    return Err(embassy_stm32::flash::Error::Size);
  }

  defmt::info!("Attempting small erase: 0x{:08X} to 0x{:08X} ({} bytes)", start, end, size);

  // Try with a very small, page-aligned region first
  flash.erase(start, end).await
}

/// Direct flash erase using register manipulation (workaround for embassy-stm32 v0.4.0 bug)
pub fn erase_sector_direct(sector_addr: u32) -> Result<(), Error> {
  defmt::info!("Direct erase sector at address: 0x{:08X}", sector_addr);

  unsafe {
    // Unlock flash
    unlock_flash();

    // Wait for any ongoing operation
    wait_flash_ready();

    // Get sector number from address
    let sector = get_sector_number(sector_addr)?;
    defmt::info!("Erasing sector {}", sector);

    // Configure sector erase
    let cr_reg = FLASH_CR as *mut u32;
    unsafe {
      let mut cr_value = cr_reg.read_volatile();
      cr_value &= !(0xF << 3); // Clear SNB bits
      cr_value |= (sector << 3) & (0xF << 3); // Set sector number
      cr_value |= FLASH_CR_SER; // Set sector erase bit
      cr_reg.write_volatile(cr_value);

      // Start erase operation
      cr_value = cr_reg.read_volatile();
      cr_value |= FLASH_CR_STRT;
      cr_reg.write_volatile(cr_value);
    }

    // Wait for completion
    wait_flash_ready();

    // Clear erase bit and lock flash
    let cr_reg = FLASH_CR as *mut u32;
    unsafe {
      let mut cr_value = cr_reg.read_volatile();
      cr_value &= !FLASH_CR_SER;
      cr_reg.write_volatile(cr_value);
    }
    lock_flash();
  }

  defmt::info!("✅ Direct sector erase completed");
  Ok(())
}

/// Direct flash write using register manipulation (workaround for embassy-stm32 v0.4.0 bug)
pub fn write_direct(addr: u32, data: &[u8]) -> Result<(), Error> {
  defmt::info!("Direct write {} bytes to address: 0x{:08X}", data.len(), addr);

  // STM32F4 supports byte programming, so no strict alignment required
  defmt::info!("Programming {} bytes starting at 0x{:08X}", data.len(), addr);

  unsafe {
    // Unlock flash
    unlock_flash();

    // Enable programming
    let cr_reg = FLASH_CR as *mut u32;
    unsafe {
      let mut cr_value = cr_reg.read_volatile();
      cr_value |= FLASH_CR_PG;
      cr_reg.write_volatile(cr_value);
    }

    // Write data byte by byte (STM32F4 supports byte programming)
    for (i, &byte) in data.iter().enumerate() {
      wait_flash_ready();

      let byte_addr = addr + i as u32;
      defmt::debug!("Writing byte {} = 0x{:02X} to address 0x{:08X}", i, byte, byte_addr);

      // Write the byte directly
      let write_ptr = byte_addr as *mut u8;
      unsafe {
        write_ptr.write_volatile(byte);
      }

      // Wait for this byte to be written
      wait_flash_ready();

      // Verify immediately after writing
      unsafe {
        let read_back = *(write_ptr as *const u8);
        if read_back != byte {
          defmt::error!("Flash write verification failed at offset {}: wrote 0x{:02X}, read 0x{:02X}", i, byte, read_back);
        } else {
          defmt::debug!("Byte {} verified OK", i);
        }
      }
    }

    // Wait for final operation and clean up
    wait_flash_ready();

    // Disable programming and lock flash
    unsafe {
      let mut cr_value = cr_reg.read_volatile();
      cr_value &= !FLASH_CR_PG;
      cr_reg.write_volatile(cr_value);
    }
    lock_flash();
  }

  defmt::info!("✅ Direct flash write completed");
  Ok(())
}

/// Helper functions for direct flash operations
unsafe fn unlock_flash() {
  let keyr_reg = FLASH_KEYR as *mut u32;
  unsafe {
    keyr_reg.write_volatile(FLASH_KEY1);
    keyr_reg.write_volatile(FLASH_KEY2);
  }
}

unsafe fn lock_flash() {
  let cr_reg = FLASH_CR as *mut u32;
  unsafe {
    let mut cr_value = cr_reg.read_volatile();
    cr_value |= FLASH_CR_LOCK;
    cr_reg.write_volatile(cr_value);
  }
}

unsafe fn wait_flash_ready() {
  let sr_reg = FLASH_SR as *const u32;
  unsafe {
    while (sr_reg.read_volatile() & FLASH_SR_BSY) != 0 {
      // Wait for flash to become ready
    }
  }
}

fn get_sector_number(addr: u32) -> Result<u32, Error> {
  // STM32F4 sector mapping
  match addr {
    0x08000000..=0x08003FFF => Ok(0), // Sector 0: 16KB
    0x08004000..=0x08007FFF => Ok(1), // Sector 1: 16KB
    0x08008000..=0x0800BFFF => Ok(2), // Sector 2: 16KB
    0x0800C000..=0x0800FFFF => Ok(3), // Sector 3: 16KB
    0x08010000..=0x0801FFFF => Ok(4), // Sector 4: 64KB
    0x08020000..=0x0803FFFF => Ok(5), // Sector 5: 128KB
    0x08040000..=0x0805FFFF => Ok(6), // Sector 6: 128KB
    0x08060000..=0x0807FFFF => Ok(7), // Sector 7: 128KB

    // STM32F413ZH additional sectors
    0x08080000..=0x0809FFFF => Ok(8),  // Sector 8: 128KB
    0x080A0000..=0x080BFFFF => Ok(9),  // Sector 9: 128KB
    0x080C0000..=0x080DFFFF => Ok(10), // Sector 10: 128KB
    0x080E0000..=0x080FFFFF => Ok(11), // Sector 11: 128KB
    0x08100000..=0x0811FFFF => Ok(12), // Sector 12: 128KB
    0x08120000..=0x0813FFFF => Ok(13), // Sector 13: 128KB
    0x08140000..=0x0815FFFF => Ok(14), // Sector 14: 128KB
    0x08160000..=0x0817FFFF => Ok(15), // Sector 15: 128KB (F413ZH)

    _ => {
      defmt::error!("Invalid flash address: 0x{:08X}", addr);
      Err(Error::Size)
    }
  }
}

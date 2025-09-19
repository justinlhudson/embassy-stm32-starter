//! Simple flash storage for STM32 using last sector
//! Provides block read/write APIs for persistent storage

use crate::board::BoardConfig;
use core::ptr;
use embassy_stm32::flash::{Error, Flash};

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
    return Err(Error::InvalidArgument);
  }
  flash.write(addr, data).await
}

/// Read a block of data from flash storage
pub fn read_block(offset: usize, buf: &mut [u8]) -> Result<(), Error> {
  let addr = storage_start() + offset as u32;
  let end_addr = addr + buf.len() as u32;
  if end_addr > storage_end() {
    defmt::error!("Attempt to read past end of storage: addr=0x{:08X}, end=0x{:08X}", addr, end_addr);
    return Err(Error::InvalidArgument);
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

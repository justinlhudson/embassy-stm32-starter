//! Persistent flash storage backed by the embassy-stm32 0.6 `Flash` driver.
//!
//! Storage region is defined per board via [`Board::FLASH_STORAGE_START`] /
//! [`Board::FLASH_STORAGE_END`]. STM32F4 word-program is 4 bytes wide; this
//! module pads write buffers up to 4-byte multiples.
//!
//! IMPORTANT: a 128 KB sector erase on STM32F4 can take ~4 seconds.
//! Perform any erase **before** calling `wdt.unleash()`. The IWDG cannot be
//! disarmed once running, which would cause a watchdog reset mid-erase.

use crate::board::{Board, BoardConfig};
use core::cell::RefCell;
use critical_section::Mutex;
use embassy_stm32::flash::{Blocking, Flash};
use embassy_stm32::peripherals::FLASH;
use embassy_stm32::Peri;

/// Local error type so callers never depend on embassy's flash error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashError {
  /// Address/length out of bounds for the storage region.
  OutOfRange,
  /// Underlying flash driver reported an error.
  Hardware,
  /// Flash driver has not been initialized via [`init`].
  Uninitialized,
}

impl From<embassy_stm32::flash::Error> for FlashError {
  fn from(_: embassy_stm32::flash::Error) -> Self {
    FlashError::Hardware
  }
}

// A single global, blocking-mode flash handle. The embassy driver owns the
// FLASH peripheral; we surface a process-wide accessor because flash ops are
// inherently a global resource on STM32F4 (single bank).
static FLASH_DRIVER: Mutex<RefCell<Option<Flash<'static, Blocking>>>> = Mutex::new(RefCell::new(None));

/// Hand the FLASH peripheral to this module. Call once at startup before any
/// `read_block`/`write_block`/`erase`.
pub fn init(p: Peri<'static, FLASH>) {
  critical_section::with(|cs| {
    *FLASH_DRIVER.borrow_ref_mut(cs) = Some(Flash::new_blocking(p));
  });
}

/// Start of the storage region (absolute flash address).
pub const fn start() -> u32 {
  BoardConfig::FLASH_STORAGE_START
}
/// End (exclusive) of the storage region.
pub const fn end() -> u32 {
  BoardConfig::FLASH_STORAGE_END
}
/// Region size in bytes.
pub const fn size() -> usize {
  BoardConfig::FLASH_STORAGE_SIZE
}

/// Read `buf.len()` bytes starting at `offset` within the storage region.
///
/// `offset` is **relative to the storage region**, not absolute.
pub fn read_block(offset: usize, buf: &mut [u8]) -> Result<(), FlashError> {
  if offset + buf.len() > size() {
    return Err(FlashError::OutOfRange);
  }
  // Memory-mapped read — works directly without a driver instance and is
  // safe on STM32F4 as long as the address is within the flash window.
  let src = (start() as usize + offset) as *const u8;
  unsafe {
    core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), buf.len());
  }
  Ok(())
}

/// Write `data` at absolute address `addr` (must be inside the storage region).
/// Length must be a multiple of 4 bytes (STM32F4 word-program). Caller is
/// responsible for ensuring the target region was erased first.
pub fn write_block(addr: u32, data: &[u8]) -> Result<(), FlashError> {
  if addr < start() || addr + data.len() as u32 > end() {
    return Err(FlashError::OutOfRange);
  }
  if !data.len().is_multiple_of(4) {
    defmt::warn!("flash write length {} not a multiple of 4; pad in caller", data.len());
    return Err(FlashError::OutOfRange);
  }
  let offset = addr; // embassy's blocking_write takes an offset from FLASH base (0x08000000)
  let offset_from_base = offset - 0x0800_0000;
  critical_section::with(|cs| {
    let mut borrow = FLASH_DRIVER.borrow_ref_mut(cs);
    let f = borrow.as_mut().ok_or(FlashError::Uninitialized)?;
    f.blocking_write(offset_from_base, data).map_err(Into::into)
  })
}

/// Erase the entire storage region. This is a long-running synchronous
/// operation (up to several seconds on STM32F4) — do it before unleashing
/// the IWDG. Despite the cost, this is a plain (non-async) blocking call
/// because the embassy STM32F4 flash driver has no async erase.
pub fn erase() -> Result<(), FlashError> {
  defmt::warn!("Flash erase 0x{:08X}..0x{:08X} (may take ~4s)...", start(), end());
  let from = start() - 0x0800_0000;
  let to = end() - 0x0800_0000;
  let result = critical_section::with(|cs| {
    let mut borrow = FLASH_DRIVER.borrow_ref_mut(cs);
    let f = borrow.as_mut().ok_or(FlashError::Uninitialized)?;
    f.blocking_erase(from, to).map_err(Into::into)
  });

  if result.is_ok() {
    let mut buf = [0u8; 16];
    if read_block(0, &mut buf).is_ok() && buf.iter().all(|&b| b == 0xFF) {
      defmt::info!("Flash erase OK (verified 0xFF prefix)");
    } else {
      defmt::error!("Flash erase verification failed: {:?}", buf);
    }
  }
  result
}

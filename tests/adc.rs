#![no_std]
#![no_main]

use defmt::assert;
use embassy_stm32_starter::hardware::adc::AdcResolution;
use embassy_stm32_starter::*;
use semihosting::process;

#[cortex_m_rt::entry]
fn main() -> ! {
  let _p = embassy_stm32::init(Default::default());

  info!("ADC test started");

  // Test resolution max values
  assert!(AdcResolution::Bits12.max_value() == 4095);
  assert!(AdcResolution::Bits16.max_value() == 65535);
  info!("✓ Resolution max values correct");

  // Test 12-bit to 16-bit conversion
  let resolution_16 = AdcResolution::Bits16;
  assert!(resolution_16.convert_from_12bit(0) == 0);
  assert!(resolution_16.convert_from_12bit(4095) == 65520); // 4095 << 4
  assert!(resolution_16.convert_from_12bit(2048) == 32768); // ~50%
  info!("✓ 12-bit to 16-bit conversion correct");

  // Test 16-bit to 12-bit conversion
  assert!(resolution_16.convert_to_12bit(0) == 0);
  assert!(resolution_16.convert_to_12bit(65520) == 4095);
  assert!(resolution_16.convert_to_12bit(32768) == 2048);
  info!("✓ 16-bit to 12-bit conversion correct");

  // Test 12-bit identity (no scaling)
  let resolution_12 = AdcResolution::Bits12;
  assert!(resolution_12.convert_from_12bit(0) == 0);
  assert!(resolution_12.convert_from_12bit(2048) == 2048);
  assert!(resolution_12.convert_from_12bit(4095) == 4095);
  info!("✓ 12-bit identity conversion correct");

  info!("ADC test completed successfully");

  process::exit(0)
}

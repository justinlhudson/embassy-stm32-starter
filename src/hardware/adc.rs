// ADC hardware abstraction for STM32
//
// This module provides ADC functionality with configurable resolution.
// STM32F4 series has 12-bit ADCs natively.
//
// The module supports:
// - 12-bit resolution (native)
// - 16-bit resolution (by left-shifting 12-bit values)
// - Reading single-shot ADC values
// - Converting raw ADC values to voltages

use embassy_stm32::adc::{Adc, AdcChannel, Resolution, SampleTime};
use embassy_stm32::peripherals::ADC1;

#[allow(unused_imports)]
use crate::prelude::*;

/// ADC resolution configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdcResolution {
  /// 12-bit resolution (0-4095) - native STM32F4 resolution
  Bits12,
  /// 16-bit resolution (0-65535) - 12-bit values left-shifted by 4
  Bits16,
}

impl AdcResolution {
  /// Get the maximum value for this resolution
  pub const fn max_value(&self) -> u16 {
    match self {
      AdcResolution::Bits12 => 4095,
      AdcResolution::Bits16 => 65535,
    }
  }

  /// Convert a 12-bit raw value to the configured resolution
  pub const fn convert_from_12bit(&self, raw_12bit: u16) -> u16 {
    match self {
      AdcResolution::Bits12 => raw_12bit,
      AdcResolution::Bits16 => raw_12bit << 4, // Left-shift by 4 to scale 12-bit to 16-bit
    }
  }

  /// Convert from configured resolution back to 12-bit
  /// (useful if you need to work with hardware 12-bit values)
  pub const fn convert_to_12bit(&self, value: u16) -> u16 {
    match self {
      AdcResolution::Bits12 => value,
      AdcResolution::Bits16 => value >> 4, // Right-shift by 4 to scale 16-bit to 12-bit
    }
  }
}

/// ADC wrapper with configurable resolution
pub struct AdcReader<'d> {
  adc: Adc<'d, ADC1>,
  resolution: AdcResolution,
  vref_mv: u16, // Reference voltage in millivolts (typically 3300 mV)
}

impl<'d> AdcReader<'d> {
  /// Create a new ADC reader with specified resolution
  ///
  /// # Arguments
  /// * `adc_peripheral` - The ADC1 peripheral
  /// * `resolution` - Desired resolution (12-bit or 16-bit)
  /// * `vref_mv` - Reference voltage in millivolts (default: 3300 mV for 3.3V)
  pub fn new(adc_peripheral: impl Into<embassy_stm32::Peri<'d, ADC1>>, resolution: AdcResolution, vref_mv: u16) -> Self {
    let mut adc = Adc::new(adc_peripheral.into());

    // STM32F4 ADC is always 12-bit at hardware level
    adc.set_resolution(Resolution::BITS12);
    adc.set_sample_time(SampleTime::CYCLES112); // Good balance between speed and accuracy

    Self { adc, resolution, vref_mv }
  }

  /// Create a new ADC reader with default settings (12-bit, 3.3V reference)
  pub fn new_default(adc_peripheral: impl Into<embassy_stm32::Peri<'d, ADC1>>) -> Self {
    Self::new(adc_peripheral, AdcResolution::Bits12, 3300)
  }

  /// Read ADC value from a channel and return in configured resolution
  pub fn read<P>(&mut self, channel: &mut P) -> u16
  where
    P: AdcChannel<ADC1>,
  {
    let raw_12bit = self.adc.blocking_read(channel);
    self.resolution.convert_from_12bit(raw_12bit)
  }

  /// Read ADC value and convert to millivolts
  pub fn read_millivolts<P>(&mut self, channel: &mut P) -> u16
  where
    P: AdcChannel<ADC1>,
  {
    let raw_value = self.read(channel);
    self.raw_to_millivolts(raw_value)
  }

  /// Convert raw ADC value to millivolts based on configured resolution
  pub fn raw_to_millivolts(&self, raw_value: u16) -> u16 {
    let max_value = self.resolution.max_value() as u32;
    ((raw_value as u32 * self.vref_mv as u32) / max_value) as u16
  }

  /// Convert millivolts to raw ADC value based on configured resolution
  pub fn millivolts_to_raw(&self, millivolts: u16) -> u16 {
    let max_value = self.resolution.max_value() as u32;
    ((millivolts as u32 * max_value) / self.vref_mv as u32) as u16
  }

  /// Get the current resolution setting
  pub fn resolution(&self) -> AdcResolution {
    self.resolution
  }

  /// Get the reference voltage in millivolts
  pub fn vref_mv(&self) -> u16 {
    self.vref_mv
  }

  /// Set the sample time for ADC conversions
  pub fn set_sample_time(&mut self, sample_time: SampleTime) {
    self.adc.set_sample_time(sample_time);
  }
}

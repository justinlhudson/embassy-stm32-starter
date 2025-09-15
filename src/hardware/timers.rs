// Busy-wait cycles per ms is now board-specific: see BoardConfig
use crate::board::BoardConfig;
#[cfg(feature = "no-rtc")]
use cortex_m::asm;
/// Timer Hardware Abstraction Layer
///
/// This module provides convenient abstractions for timer operations
/// and timing utilities for the STM32F446RE microcontroller.
#[cfg(not(feature = "no-rtc"))]
use embassy_time::Timer;

/// Common timing utilities and constants
pub struct TimingUtils;

impl TimingUtils {
  /// Standard blink rates in milliseconds
  pub const FAST_BLINK_MS: u64 = 100;
  pub const MEDIUM_BLINK_MS: u64 = 250;
  pub const SLOW_BLINK_MS: u64 = 500;
  pub const VERY_SLOW_BLINK_MS: u64 = 1000;

  /// Button debounce delay
  pub const BUTTON_DEBOUNCE_MS: u64 = 50;

  /// Watchdog pet interval (25% of board's watchdog timeout, in ms)
  pub fn watchdog_pet_ms() -> u64 {
    // BoardConfig::WATCHDOG_TIMEOUT_US is in microseconds
    (((BoardConfig::WATCHDOG_TIMEOUT_US as u64 / 4) / 1000).max(1u64)) as u64
  }

  /// Heartbeat interval
  pub const HEARTBEAT_INTERVAL_MS: u64 = 2000;

  /// RTC update interval
  pub const RTC_UPDATE_INTERVAL_MS: u64 = 1000;

  /// Board-agnostic delay in milliseconds
  #[cfg(not(feature = "no-rtc"))]
  pub async fn delay_ms(ms: u64) {
    Timer::after_millis(ms).await;
  }

  /// For Blue Pill (STM32F1, no async timer driver): busy-wait blocking delay
  #[cfg(feature = "no-rtc")]
  pub async fn delay_ms(ms: u64) {
    // 72 MHz, simple busy-wait loop: 1 ms ≈ 72_000 cycles
    // This is not precise, but enough for basic delays
    for _ in 0..ms {
      for _ in 0..BoardConfig::cycles_per_ms() {
        asm::nop();
      }
    }
  }
}

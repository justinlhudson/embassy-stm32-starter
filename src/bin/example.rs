#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32_starter::board::BoardConfig;
use embassy_stm32_starter::common::tasks::*;
#[allow(unused_imports)]
use embassy_stm32_starter::prelude::*;
use embassy_stm32_starter::*;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
  info!("Example starting...");

  // Log board configuration info
  info!("Running on {}", BoardConfig::BOARD_NAME);
  info!(
    "MCU: {} with {}KB flash, {}KB RAM",
    BoardConfig::MCU_NAME,
    BoardConfig::FLASH_SIZE_KB,
    BoardConfig::RAM_SIZE_KB
  );
  info!("LED: {} ({})", BoardConfig::LED_PIN_NAME, BoardConfig::LED_DESCRIPTION);
  info!(
    "Button: {} ({})",
    BoardConfig::BUTTON_PIN_NAME,
    BoardConfig::BUTTON_DESCRIPTION
  );

  let config = Config::default();
  let p = embassy_stm32::init(config);
  let (led, button, mut wdt, rtc, comm) = BoardConfig::init_all_hardware(_spawner, p);

  _spawner
    .spawn(led_blink(led, hardware::timers::TimingUtils::FAST_BLINK_MS))
    .ok();
  _spawner.spawn(button_monitor(button)).ok();
  _spawner.spawn(rtc_clock(rtc)).ok();

  // Spawn comms task (handles RX/echo separately)
  _spawner.spawn(comm_task(comm)).ok();

  info!("U ready? U an't ready!");
  loop {
    // Keep feeding the watchdog from the main loop only
    wdt.pet();
    Timer::after_millis(hardware::timers::TimingUtils::WATCHDOG_PET_MS).await;
  }
}

// Handle comms (receive frames and echo Ping)
#[embassy_executor::task]
async fn comm_task(mut tx: embassy_stm32::usart::UartTx<'static, embassy_stm32::mode::Async>) {
  loop {
    if let Some(msg) = embassy_stm32_starter::service::comm::read() {
      info!(
        "message: command={}, {}/{} bytes",
        msg.command,
        msg.payload.len(),
        embassy_stm32_starter::service::comm::COMMS_MAX_PAYLOAD
      );

      if core::convert::TryFrom::try_from(msg.command) == Ok(embassy_stm32_starter::service::comm::Command::Ping) {
        let mut tx_ref = &mut tx;
        embassy_stm32_starter::service::comm::write(&mut tx_ref, &msg);
      }
    } else {
      // Small backoff when no message is ready
      Timer::after_millis(10).await;
    }
  }
}

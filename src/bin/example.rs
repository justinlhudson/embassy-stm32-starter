#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32_starter::board::BoardConfig;
use embassy_stm32_starter::common::tasks::*;
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
  let (led, button, mut wdt, rtc, mut comm) = BoardConfig::init_all_hardware(_spawner, p);

  _spawner
    .spawn(led_blink(led, hardware::timers::TimingUtils::FAST_BLINK_MS))
    .ok();
  _spawner.spawn(button_monitor(button)).ok();
  _spawner.spawn(rtc_clock_task(rtc)).ok();

  info!("U ready? U an't ready!");
  loop {
    if let Some(msg) = embassy_stm32_starter::service::comm::read() {
      info!(
        "Comms msg: command={}, {}/{} bytes",
        msg.command,
        msg.payload.len(),
        embassy_stm32_starter::service::comm::COMMS_MAX_PAYLOAD
      );

      if msg.command == 0x03 {
        let mut tx_ref = &mut comm;
        embassy_stm32_starter::service::comm::write(&mut tx_ref, &msg);
      }
    }

    // Feed watchdog and sleep below the configured timeout (~1s)
    wdt.pet();
    Timer::after_millis(hardware::timers::TimingUtils::WATCHDOG_PET_MS).await;
  }
}

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
  info!("Button: {} ({})", BoardConfig::BUTTON_PIN_NAME, BoardConfig::BUTTON_DESCRIPTION);

  let config = Config::default();
  let p = embassy_stm32::init(config);
  let (led, button, mut wdt, rtc, comm) = BoardConfig::init_all_hardware(_spawner, p);

  _spawner.spawn(led_blink(led, hardware::timers::TimingUtils::FAST_BLINK_MS)).ok();
  _spawner.spawn(button_monitor(button)).ok();
  _spawner.spawn(rtc_clock(rtc)).ok();
  _spawner.spawn(comm_task(comm)).ok();

  info!("U ready? U an't ready!");
  let mut last_sp: u32 = 0;
  // Set this to your RAM end address (from linker script or BoardConfig)
  let ram_end: u32 = 0x20020000; // Example: 128KB RAM ends at 0x20020000
  loop {
    wdt.pet(); // pet the watchdog from main loop
    // Print stack usage in KB only if changed
    let sp: u32;
    unsafe { core::arch::asm!("mov {}, sp", out(reg) sp) }
    if sp > last_sp {
      let stack_used = ram_end.saturating_sub(sp);
      let stack_used_kb = stack_used / 1024;
      info!("Stack used: {} KB (SP: {=u32:x})", stack_used_kb, sp);
      last_sp = sp;
    }
    Timer::after_millis(hardware::timers::TimingUtils::WATCHDOG_PET_MS).await;
  }
}

#[embassy_executor::task]
async fn comm_task(mut tx: embassy_stm32::usart::UartTx<'static, embassy_stm32::mode::Async>) {
  loop {
    if let Some(msg) = embassy_stm32_starter::service::comm::read() {
      info!(
        "message (rx): command={}, {}/{} bytes",
        msg.command,
        msg.payload.len(),
        embassy_stm32_starter::service::comm::COMMS_MAX_PAYLOAD
      );
      // Handle command(s) here
      // For now, just echo pings back
      if core::convert::TryFrom::try_from(msg.command) == Ok(embassy_stm32_starter::service::comm::Command::Ping) {
        let mut tx_ref = &mut tx;
        embassy_stm32_starter::service::comm::write(&mut tx_ref, &msg);
        info!(
          "message (tx): command={}, {}/{} bytes",
          msg.command,
          msg.payload.len(),
          embassy_stm32_starter::service::comm::COMMS_MAX_PAYLOAD
        );
      }
    } else {
      Timer::after_millis(10).await; // backoff when no message is ready
    }
  }
}

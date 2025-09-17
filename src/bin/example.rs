#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32_starter::board::BoardConfig;
use embassy_stm32_starter::common::tasks::*;
use embassy_stm32_starter::hardware::Timing;
#[allow(unused_imports)]
use embassy_stm32_starter::prelude::*;
use embassy_stm32_starter::*;

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

  _spawner.spawn(button_monitor(button)).ok();
  _spawner.spawn(rtc_clock(rtc)).ok();
  _spawner.spawn(comm_task(comm, led)).ok();

  info!("U ready? U an't ready!");
  let mut last_sp: u32 = 0;
  loop {
    // Print stack usage in KB only if changed
    let sp: u32;
    unsafe { core::arch::asm!("mov {}, sp", out(reg) sp) }
    if sp > last_sp {
      let stack_used = sp.saturating_sub(BoardConfig::RAM_START);
      let stack_used_kb = (stack_used as u32) / 1024; // Explicitly cast stack_used to u32 before division to ensure no implicit type promotion
      let stack_left = BoardConfig::RAM_END.saturating_sub(sp);
      let stack_left_kb = stack_left / 1024;
      info!("Stack used: {}/{} KB (SP: {=u32:x})", stack_used_kb, stack_used_kb + stack_left_kb, sp);
      last_sp = sp;
    }

    wdt.pet(); // if got no data pet the dog
    Timing::delay_ms(Timing::WATCHDOG_PET_MS).await;
  }
}

#[embassy_executor::task]
async fn comm_task(mut tx: embassy_stm32::usart::UartTx<'static, embassy_stm32::mode::Async>, mut led: embassy_stm32::gpio::Output<'static>) {
  let mut last_fcs_error_count = 0u8;
  loop {
    // Try to read a message; if FCS error occurred, log it
    match embassy_stm32_starter::service::comm::read() {
      Some(msg) => {
        led.set_high(); // Turn on the LED when a message is received

        debug!("Received message: command=0x{:04x}, id={}, len={}", msg.command, msg.id, msg.length);

        // *** Handle command(s) here *** //
        if core::convert::TryFrom::try_from(msg.command) == Ok(embassy_stm32_starter::service::comm::Command::Ping) {
          debug!("Sending Ping response");
          let mut tx_ref = &mut tx;
          embassy_stm32_starter::service::comm::write(&mut tx_ref, &msg);
        }
      }
      None => {
        // Could be no message, or FCS error (already logged in comm.rs)
        led.set_low(); // Turn off the LED when no message is received
        let fcs_errors = embassy_stm32_starter::service::comm::fcs_error_count();
        if fcs_errors != last_fcs_error_count {
          debug!("HDLC FCS error count: {}", fcs_errors);
          last_fcs_error_count = fcs_errors;
        }
        Timer::after_millis(1).await; // backoff when no message is ready
      }
    }
  }
}

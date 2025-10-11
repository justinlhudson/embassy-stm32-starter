#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32_starter::board::BoardConfig;
use embassy_stm32_starter::common::tasks::*;
use embassy_stm32_starter::hardware::Timing;
use embassy_stm32_starter::hardware::flash;
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

  // Demonstrate flash storage functionality
  flash_demo().await;

  _spawner.spawn(button_monitor(button)).ok();
  _spawner.spawn(rtc_clock(rtc)).ok();
  _spawner.spawn(comm_task(comm, led)).ok();

  info!("U ready? U ain't ready!");
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

    wdt.pet();
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
        // *** Handle command(s) here *** //
        if core::convert::TryFrom::try_from(msg.command) == Ok(embassy_stm32_starter::service::comm::Command::Ping) {
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

/// Demonstrate flash storage by reading previous random number and writing a new one
async fn flash_demo() {
  info!("🔥 Flash Storage Demo - Auto-erase on dirty flash");

  // Read current flash contents
  let mut buffer = [0u8; 16];
  flash::read_block(0, &mut buffer).unwrap();
  info!("📖 Current flash contents: {:?}", buffer);

  // Check if flash is erased (all 0xFF)
  if buffer[0..4].iter().all(|&b| b == 0xFF) {
    // Flash is clean - write test data
    let data = [0x12, 0x34, 0x56, 0x78];
    flash::write_block(flash::start(), &data).unwrap();
    info!("✅ Successfully wrote {:?} to clean flash", data);
  } else {
    // Flash has data - erase it for next boot
    info!("⚠️  Flash contains data - erasing for next boot");
    flash::erase().await.unwrap();
    info!("🔄 Flash erased! On next boot, demo will write to clean flash");
  }
}

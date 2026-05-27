#![no_std]
#![no_main]

//! Example application: showcases the full stack — board init, flash demo,
//! watchdog, RTC, button monitor, and HDLC comms (Ping / Version handled
//! automatically by `comm::dispatch`).

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use embassy_stm32_starter::common::tasks::{button_monitor, rtc_clock};
use embassy_stm32_starter::hardware::flash;
use embassy_stm32_starter::prelude::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  info!("Example starting...");
  info!("Running on {}", BoardConfig::BOARD_NAME);
  info!(
    "MCU: {} with {}KB flash, {}KB RAM",
    BoardConfig::MCU_NAME,
    BoardConfig::FLASH_SIZE_KB,
    BoardConfig::RAM_SIZE_KB
  );
  info!("LED: {} ({})", BoardConfig::LED_PIN_NAME, BoardConfig::LED_DESCRIPTION);
  info!("Button: {} ({})", BoardConfig::BUTTON_PIN_NAME, BoardConfig::BUTTON_DESCRIPTION);

  let p = embassy_stm32::init(Config::default());
  let BoardHardware { led, button, mut wdt, rtc, tx } = BoardConfig::init(spawner, p);

  // Flash erase can take ~4s on STM32F4 — do it before unleashing the IWDG.
  flash_demo();
  wdt.unleash();

  // Spawn the HDLC consumer (parses incoming serial bytes into Messages).
  comm::start(spawner);

  spawner.spawn(button_monitor(button).unwrap());
  spawner.spawn(rtc_clock(rtc).unwrap());
  spawner.spawn(comm_task(tx, led).unwrap());

  info!("U ready? U ain't ready!");
  let mut last_sp: u32 = 0;
  loop {
    let sp: u32;
    unsafe { core::arch::asm!("mov {}, sp", out(reg) sp) }
    if sp > last_sp {
      let used_kb = sp.saturating_sub(BoardConfig::RAM_START) / 1024;
      let left_kb = BoardConfig::RAM_END.saturating_sub(sp) / 1024;
      info!("Stack: {}/{} KB (SP: {=u32:x})", used_kb, used_kb + left_kb, sp);
      last_sp = sp;
    }
    wdt.pet();
    Timing::delay_ms(Timing::WATCHDOG_PET_MS).await;
  }
}

#[embassy_executor::task]
async fn comm_task(mut tx: UartTx<'static, Async>, mut led: Output<'static>) {
  let mut last_fcs = 0u8;
  loop {
    let msg = comm::recv().await;
    led.set_high();
    // Built-in Ping echo + Version reply; everything else falls through to
    // the closure (no app-specific commands in this example).
    comm::dispatch(&mut tx, &msg, |_msg, _tx| {});
    led.set_low();

    let fcs = comm::fcs_error_count();
    if fcs != last_fcs {
      debug!("HDLC FCS error count: {}", fcs);
      last_fcs = fcs;
    }
  }
}

/// Demonstrate flash storage: read first 16 bytes, then either write a test
/// pattern (if clean) or erase (if dirty) so the next boot can write again.
fn flash_demo() {
  info!("Flash storage demo");
  let mut buf = [0u8; 16];
  flash::read_block(0, &mut buf).unwrap();
  info!("Current flash[0..16]: {:?}", buf);

  if buf[0..4].iter().all(|&b| b == 0xFF) {
    let data = [0x12, 0x34, 0x56, 0x78];
    flash::write_block(flash::start(), &data).unwrap();
    info!("Wrote {:?} to clean flash", data);
  } else {
    info!("Flash dirty — erasing (~4s)...");
    flash::erase().unwrap();
    info!("Erased. Next boot will write test pattern.");
  }
}

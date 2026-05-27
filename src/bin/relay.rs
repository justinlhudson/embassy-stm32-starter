#![no_std]
#![no_main]

//! Relay application: HDLC-controlled GPIO toggle on PA9 (Arduino D8 on
//! Nucleo-F446RE). Press the user button to toggle D8 locally; send a
//! `Raw` command with payload `[0xD8, 0|1]` to drive it from the host.
//!
//! Policy: this app resets the MCU on persistent HDLC FCS errors (a
//! workaround for a noisy link). The library never resets on its own.

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use embassy_stm32_starter::hardware::GpioDefaults;
use embassy_stm32_starter::prelude::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  info!("Relay app starting on {}", BoardConfig::BOARD_NAME);

  let p = embassy_stm32::init(Config::default());
  let BoardHardware { led, button, mut wdt, rtc: _, tx } = BoardConfig::init(spawner, p);

  // Grab an extra GPIO not exposed by the board abstraction. `steal()` is
  // the standard embassy escape hatch; safe here because no other code
  // owns PA9 (board init touched only the pins listed in its module doc).
  let extra = unsafe { embassy_stm32::Peripherals::steal() };
  let d8 = Output::new(extra.PA9, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);
  wdt.unleash(); // no flash ops here

  comm::start(spawner);
  spawner.spawn(operation_task(tx, led, d8, button).unwrap());

  loop {
    wdt.pet();
    Timing::delay_ms(Timing::WATCHDOG_PET_MS).await;
  }
}

#[embassy_executor::task]
async fn operation_task(
  mut tx: UartTx<'static, Async>,
  mut led: Output<'static>,
  mut d8: Output<'static>,
  button: Input<'static>,
) {
  d8.set_low();
  let last_fcs = 0u8;
  let mut btn_state = button.is_high();

  loop {
    // Debounced button edge -> toggle D8 on press.
    let cur = button.is_high();
    if cur != btn_state {
      Timer::after_millis(Timing::BUTTON_DEBOUNCE_MS).await;
      if button.is_high() == cur {
        btn_state = cur;
        if btn_state {
          d8.toggle();
        }
      }
    }

    if let Some(msg) = comm::read() {
      led.set_high();
      comm::dispatch(&mut tx, &msg, |m, _tx| {
        if core::convert::TryFrom::try_from(m.command) == Ok(comm::Command::Raw)
          && m.payload.len() >= 2
          && m.payload[0] == 0xD8
        {
          match m.payload[1] {
            1 => { info!("D8 <- HIGH"); d8.set_high(); }
            0 => { info!("D8 <- LOW");  d8.set_low(); }
            other => info!("D8: unknown value {} (ignored)", other),
          }
        }
      });
      led.set_low();
    } else {
      led.set_low();
      Timer::after_millis(1).await;
    }

    // App-level policy: bounce the MCU on persistent FCS errors.
    let fcs = comm::fcs_error_count();
    if fcs != last_fcs {
      debug!("HDLC FCS errors: {}", fcs);
      let _ = last_fcs; // sys_reset never returns; quiet `unused_assignments`
      cortex_m::peripheral::SCB::sys_reset();
    }
  }
}

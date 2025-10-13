#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::Output;
use embassy_stm32_starter::board::BoardConfig;
use embassy_stm32_starter::hardware::{GpioDefaults, Timing};
use embassy_stm32_starter::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  info!("Relay app starting");
  info!("Board: {}", BoardConfig::BOARD_NAME);

  let p = embassy_stm32::init(Config::default());
  let (led, button, mut wdt, _rtc, comm) = BoardConfig::init_all_hardware(spawner, p);

  // Create D8 output (Arduino D8 = PA9 on Nucleo-F446RE)
  let p2 = unsafe { embassy_stm32::Peripherals::steal() };
  let d8 = Output::new(p2.PA9, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);

  spawner.spawn(operation_task(comm, led, d8, button)).ok();

  loop {
    wdt.pet();
    Timing::delay_ms(Timing::WATCHDOG_PET_MS).await;
  }
}

#[embassy_executor::task]
async fn operation_task(
  mut tx: embassy_stm32::usart::UartTx<'static, embassy_stm32::mode::Async>,
  mut led: embassy_stm32::gpio::Output<'static>,
  mut d8: embassy_stm32::gpio::Output<'static>,
  mut button: embassy_stm32::gpio::Input<'static>,
) {
  let mut last_fcs = 0u8;
  d8.set_low();
  let mut btn_state = button.is_high();
  loop {
    // Debounced button edge: on press, toggle D8
    let cur = button.is_high();
    if cur != btn_state {
      Timer::after_millis(Timing::BUTTON_DEBOUNCE_MS).await;
      let confirm = button.is_high();
      if confirm == cur {
        btn_state = cur;
        if btn_state {
          d8.toggle();
        }
      }
    }
    match embassy_stm32_starter::service::comm::read() {
      Some(msg) => {
        led.set_high();
        if core::convert::TryFrom::try_from(msg.command) == Ok(embassy_stm32_starter::service::comm::Command::Ping) {
          let mut tx_ref = &mut tx;
          embassy_stm32_starter::service::comm::write(&mut tx_ref, &msg);
        } else if core::convert::TryFrom::try_from(msg.command) == Ok(embassy_stm32_starter::service::comm::Command::Raw) {
          if msg.payload.len() >= 2 && msg.payload[0] == 0xD8 {
            match msg.payload[1] {
              1 => {
                info!("D8 command: HIGH (from comms)");
                d8.set_high()
              }
              0 => {
                info!("D8 command: LOW (from comms)");
                d8.set_low()
              }
              other => {
                info!("D8 command: unknown value {} (ignored)", other);
              }
            }
          }
        }
      }
      None => {
        led.set_low();
        let fcs = embassy_stm32_starter::service::comm::fcs_error_count();
        if fcs != last_fcs {
          debug!("HDLC FCS errors: {}", fcs);
          last_fcs = fcs;
          cortex_m::peripheral::SCB::sys_reset();
        }
        Timer::after_millis(1).await;
      }
    }
  }
}

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32_starter::board::BoardConfig;
use embassy_stm32_starter::common::tasks::*;
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
  let (led, button, _wdt, rtc, mut comm) = BoardConfig::init_all_hardware(_spawner, p);

  use embedded_io::Write as _;
  let _ = comm.write_all(b"Hello from Embassy USART2!\r\n");
  let _ = comm.flush();

  info!("Hardware setup complete; serial RX and HDLC tasks running");

  info!("Spawning system tasks...");

  _spawner
    .spawn(led_blink_custom(led, hardware::timers::TimingUtils::FAST_BLINK_MS))
    .ok();
  _spawner.spawn(button_monitor(button)).ok();
  _spawner.spawn(rtc_clock_task(rtc)).ok();

  info!("All tasks spawned, starting main loop");

  loop {
    if let Some(msg) = embassy_stm32_starter::hardware::serial::read() {
      info!("UART RX ({} bytes)", msg.len());
      let _ = comm.write_all(&msg);
      let _ = comm.write_all(b"\r\n");
      let _ = comm.flush();
    }

    if let Some(frame) = embassy_stm32_starter::service::comm::try_read_decoded_hdlc() {
      info!("HDLC frame ({} bytes)", frame.len());
    }

    if let Some(msg) = embassy_stm32_starter::service::comm::try_read_comms_msg() {
      info!(
        "Comms msg: command={}, id=0x{:08x}, {}/{} bytes",
        msg.command,
        msg.id,
        msg.payload.len(),
        embassy_stm32_starter::service::comm::COMMS_MAX_PAYLOAD
      );

      if msg.command == 0x03 {
        let mut tx_ref = &mut comm;
        embassy_stm32_starter::service::comm::write_comms_frame(
          &mut tx_ref,
          msg.command,
          msg.id,
          msg.fragments,
          msg.fragment,
          &msg.payload,
        );
      }
    }

    info!("Main loop - waiting (TODO: implement watchdog)");
    Timer::after_millis(1000).await;
  }
}

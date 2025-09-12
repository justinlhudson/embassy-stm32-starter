#![no_std]
#![no_main]

//! Blinky Application
//! 
//! This application demonstrates basic Embassy functionality with LED blinking,
//! button monitoring, RTC clock, and system heartbeat.
//!
//! ## Board Support
//! Pin configurations are handled by the board.rs file (copied from boards/* by setup.sh)

use embassy_executor::Spawner;
// use embassy_stm32::gpio::{Input, Output};
// use embassy_stm32::rtc::Rtc;
use embassy_stm32::Config;
// Serial is initialized through BoardConfig::init_serial
use embassy_time::Timer;
use embassy_stm32_starter::*;
use embassy_stm32_starter::common::tasks::*;
// use embassy_stm32_starter::hardware::TimingUtils;

// Select board configuration from the board.rs file (copied by setup.sh)
use embassy_stm32_starter::board::BoardConfig;

// /// Spawn common system tasks for the blinky application
// fn spawn_system_tasks(
//     spawner: &Spawner,
//     led: Output<'static>,
//     button: Input<'static>,
//     rtc: Rtc,
// ) -> Result<(), embassy_executor::SpawnError> {
//     spawner.spawn(led_blink_custom(led, TimingUtils::FAST_BLINK_MS))?;
//     spawner.spawn(button_monitor(button))?;
//     spawner.spawn(heartbeat_task())?;
//     spawner.spawn(rtc_clock_task(rtc))?;
//     Ok(())
// }

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Embassy blinky starting...");
    
    // Log board configuration info
    info!("Running on {}", BoardConfig::BOARD_NAME);
    info!("MCU: {} with {}KB flash, {}KB RAM", 
          BoardConfig::MCU_NAME, 
          BoardConfig::FLASH_SIZE_KB, 
          BoardConfig::RAM_SIZE_KB);
    info!("LED: {} ({})", 
          BoardConfig::LED_PIN_NAME, 
          BoardConfig::LED_DESCRIPTION);
    info!("Button: {} ({})", 
          BoardConfig::BUTTON_PIN_NAME, 
          BoardConfig::BUTTON_DESCRIPTION);
    
    // Initialize hardware
            let config = Config::default();
            let p = embassy_stm32::init(config);
    
      info!("Peripherals initialized");
    
                  // Initialize all hardware via board config (LED, button, WDT, RTC, serial)
                  let (led, button, _wdt, rtc, mut uart_tx) = BoardConfig::init_all_hardware(_spawner, p);

      // Send a hello message over serial
      use embedded_io::Write as _;
      let _ = uart_tx.write_all(b"Hello from Embassy USART2!\r\n");
      let _ = uart_tx.flush();

            info!("Hardware setup complete; serial RX and HDLC tasks running");
    
    info!("Spawning system tasks...");
    
            // Spawn system tasks
            _spawner.spawn(led_blink_custom(led, hardware::timers::TimingUtils::FAST_BLINK_MS)).ok();
            _spawner.spawn(button_monitor(button)).ok();
            _spawner.spawn(rtc_clock_task(rtc)).ok();

    info!("All tasks spawned, starting main loop");
    
            // Main task loop: periodically echo any received lines, send a sample HDLC frame, and tick
    loop {
            // Non-blocking read from serial queue (raw bytes)
            if let Some(msg) = embassy_stm32_starter::hardware::serial::read() {
                  info!("UART RX ({} bytes)", msg.len());
                  // Echo back
                  let _ = uart_tx.write_all(&msg);
                  let _ = uart_tx.write_all(b"\r\n");
                  let _ = uart_tx.flush();
            }

            // Example: also drain decoded HDLC frames if present
            if let Some(frame) = embassy_stm32_starter::service::comm::try_read_decoded_hdlc() {
                  info!("HDLC frame ({} bytes)", frame.len());
            }

            // Example: drain parsed Comms messages if present
            if let Some(msg) = embassy_stm32_starter::service::comm::try_read_comms_msg() {
                  info!(
                        "Comms msg: command={}, id=0x{:08x}, {}/{} bytes",
                        msg.command,
                        msg.id,
                        msg.payload.len(),
                        embassy_stm32_starter::service::comm::COMMS_MAX_PAYLOAD
                  );
            }

                  // Periodically send a small HDLC-framed payload
                  {
                        // Send a Comms-framed payload over HDLC
                        static mut MSG_ID_COUNTER: u32 = 0;
                        // Safety: only accessed in this single-threaded async context
                        let msg_id = unsafe {
                              let id = MSG_ID_COUNTER;
                              MSG_ID_COUNTER = MSG_ID_COUNTER.wrapping_add(1);
                              id
                        };
                        let mut tx_ref = &mut uart_tx;
                        embassy_stm32_starter::service::comm::write_comms_frame(
                              &mut tx_ref,
                              1,          // command
                              msg_id,     // id
                              1,          // fragments
                              0,          // fragment index
                              b"ping",
                        );
                  }

            info!("Main loop - waiting (TODO: implement watchdog)");
        Timer::after_millis(1000).await;
    }
}
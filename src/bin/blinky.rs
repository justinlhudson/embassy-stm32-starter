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
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::rtc::Rtc;
use embassy_stm32::Config;
use embassy_time::Timer;
use embassy_stm32_starter::*;
use embassy_stm32_starter::common::tasks::*;
use embassy_stm32_starter::hardware::TimingUtils;

// Select board configuration from the board.rs file (copied by setup.sh)
use embassy_stm32_starter::board::{BoardConfig, BoardConfiguration};

/// Spawn common system tasks for the blinky application
fn spawn_system_tasks(
    spawner: &Spawner,
    led: Output<'static>,
    button: Input<'static>,
    rtc: Rtc,
) -> Result<(), embassy_executor::SpawnError> {
    spawner.spawn(led_blink_custom(led, TimingUtils::FAST_BLINK_MS))?;
    spawner.spawn(button_monitor(button))?;
    spawner.spawn(heartbeat_task())?;
    spawner.spawn(rtc_clock_task(rtc))?;
    Ok(())
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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
    let peripherals = embassy_stm32::init(config);
    
    info!("Peripherals initialized");
    
    // Initialize all hardware using board-specific configuration
    let (led, button, mut wdt, rtc, mut serial) = BoardConfig::init_all_hardware(peripherals);

    // Send a hello message over serial using blocking write
    let _ = serial.write(b"Hello from Embassy USART2!\r\n");

    info!("Hardware setup complete");
    
    info!("Spawning system tasks...");
    
    // Spawn all system tasks
    spawn_system_tasks(&spawner, led, button, rtc)
        .expect("Failed to spawn system tasks");

    info!("All tasks spawned, starting watchdog management");
    
    // Main task manages the watchdog
    loop {
        info!("Main loop - petting watchdog");
        wdt.pet();
        Timer::after_millis(250).await; // Pet every 250ms (25% of 1000ms timeout)
    }
}
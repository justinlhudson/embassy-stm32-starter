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
  info!("🔥 Flash Storage Demo");

  // Flash storage layout:
  // Offset 0-3: Magic header (0xDEADBEEF)
  // Offset 4-7: Random number (u32)
  // Offset 8-11: Boot counter (u32)

  let magic_header: u32 = 0xDEADBEEF;
  let mut read_buffer = [0u8; 12]; // Read 12 bytes (magic + random + counter)

  // Read existing data from flash
  info!("📖 Reading from flash storage...");
  match flash::read_block(0, &mut read_buffer) {
    Ok(()) => {
      // Parse the data
      let stored_magic = u32::from_le_bytes([read_buffer[0], read_buffer[1], read_buffer[2], read_buffer[3]]);
      let stored_random = u32::from_le_bytes([read_buffer[4], read_buffer[5], read_buffer[6], read_buffer[7]]);
      let stored_counter = u32::from_le_bytes([read_buffer[8], read_buffer[9], read_buffer[10], read_buffer[11]]);

      if stored_magic == magic_header {
        info!("✅ Valid data found in flash!");
        info!("   Previous random number: 0x{:08X} ({})", stored_random, stored_random);
        info!("   Boot counter: {}", stored_counter);

        // Generate new random number (simple PRNG based on previous value and counter)
        let new_random = generate_pseudo_random(stored_random, stored_counter);
        let new_counter = stored_counter.wrapping_add(1);

        // Write new data to flash
        write_flash_data(magic_header, new_random, new_counter).await;
      } else {
        info!("🆕 No valid data found - initializing flash storage");
        let initial_random = generate_pseudo_random(0x12345678, 0);
        write_flash_data(magic_header, initial_random, 1).await;
      }
    }
    Err(_) => {
      warn!("❌ Failed to read from flash storage");
    }
  }
}

/// Write magic header, random number, and boot counter to flash
async fn write_flash_data(magic: u32, random: u32, counter: u32) {
  info!("💾 Writing to flash storage...");
  info!("   New random number: 0x{:08X} ({})", random, random);
  info!("   Boot counter: {}", counter);

  // Prepare data buffer
  let mut write_buffer = [0u8; 12];
  write_buffer[0..4].copy_from_slice(&magic.to_le_bytes());
  write_buffer[4..8].copy_from_slice(&random.to_le_bytes());
  write_buffer[8..12].copy_from_slice(&counter.to_le_bytes());

  // Check if we need to erase by reading current data
  let mut current_data = [0u8; 12];
  if flash::read_block(0, &mut current_data).is_ok() {
    // Check if all bytes in the region are 0xFF (erased state)
    let needs_erase = !current_data.iter().all(|&b| b == 0xFF);

    if needs_erase {
      warn!("⚠️  Flash erase needed but skipping due to system stability");
      warn!("   This is a limitation of running flash erase operations from flash memory");
      warn!("   In a real application, you would:");
      warn!("   1. Move erase/write code to RAM, or");
      warn!("   2. Use a bootloader for flash updates, or");
      warn!("   3. Only write to erased regions");
      info!("   Current flash content: {:?}", current_data);
      return;
    }
  }

  // Only write if flash is already erased (all 0xFF)
  info!("✅ Flash already erased, proceeding with write...");
  match flash::write_direct(flash::storage_start(), &write_buffer) {
    Ok(()) => {
      info!("✅ Data written to flash successfully");

      // Verify the write
      let mut verify_buffer = [0u8; 12];
      if flash::read_block(0, &mut verify_buffer).is_ok() && verify_buffer == write_buffer {
        info!("✅ Flash write verified - data matches!");
      } else {
        warn!("❌ Flash write verification failed");
      }
    }
    Err(_) => {
      warn!("❌ Failed to write data to flash");
    }
  }
}

/// Simple pseudo-random number generator
fn generate_pseudo_random(seed: u32, iteration: u32) -> u32 {
  // Linear congruential generator with system timer influence
  let mut rng = seed.wrapping_mul(1664525).wrapping_add(1013904223);
  rng = rng.wrapping_add(iteration.wrapping_mul(2654435761)); // Add counter influence

  // Add some "entropy" from the current time (rough estimation)
  // In a real application, you might use a proper RNG or hardware entropy
  rng = rng.wrapping_add((iteration & 0xFF).wrapping_mul(7919));

  rng
}

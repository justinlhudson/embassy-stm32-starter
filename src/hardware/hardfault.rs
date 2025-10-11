use cortex_m_rt::exception;
use defmt_rtt as _;

/// Performs a system reset via the System Control Block (SCB)
unsafe fn system_reset() -> ! {
  const SCB_AIRCR: *mut u32 = 0xE000_ED0C as *mut u32;
  const AIRCR_VECTKEY: u32 = 0x05FA_0000;
  const AIRCR_SYSRESETREQ: u32 = 1 << 2;

  // Write to Application Interrupt and Reset Control Register
  unsafe {
    core::ptr::write_volatile(SCB_AIRCR, AIRCR_VECTKEY | AIRCR_SYSRESETREQ);
  }

  // Wait for reset (should not reach here)
  loop {}
}

#[exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
  // Print core registers from the exception frame
  let regs = ef as *const _ as *const u32;
  defmt::error!("HardFault! ExceptionFrame registers:");
  unsafe {
    defmt::error!(" r0   = {=u32:x}", *regs.offset(0));
    defmt::error!(" r1   = {=u32:x}", *regs.offset(1));
    defmt::error!(" r2   = {=u32:x}", *regs.offset(2));
    defmt::error!(" r3   = {=u32:x}", *regs.offset(3));
    defmt::error!(" r12  = {=u32:x}", *regs.offset(4));
    defmt::error!(" lr   = {=u32:x}", *regs.offset(5));
    defmt::error!(" pc   = {=u32:x}", *regs.offset(6));
    defmt::error!(" xpsr = {=u32:x}", *regs.offset(7));
    // Print the last instruction (16-bit at PC)
    let pc = *regs.offset(6);
    let instr = core::ptr::read_volatile(pc as *const u16);
    defmt::error!("Last instruction (16-bit at PC): {=u16:x}", instr);
  }

  defmt::error!("Performing automatic system reset in 100ms...");

  // Short delay to allow log output to be transmitted
  for _ in 0..1_000_000 {
    cortex_m::asm::nop();
  }

  // Automatically reset the system
  unsafe {
    system_reset();
  }
}

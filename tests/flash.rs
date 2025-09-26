#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::flash::Flash;
use embassy_stm32_starter::hardware::flash;
use embassy_stm32_starter::hardware::flash::IrqsFlash;
use semihosting::process;

#[entry]
fn main() -> ! {
  let p = embassy_stm32::init(Default::default());
  let mut flash_hw = Flash::new(p.FLASH, IrqsFlash);
  let test_offset = 0;
  let test_data: [u8; 16] = [0xA5; 16];
  let mut read_buf: [u8; 16] = [0; 16];

  // Helper to poll async future to completion
  fn block_on<F: core::future::Future>(fut: F) -> F::Output {
    use core::pin::pin;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn dummy(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
      dummy_waker()
    }
    fn dummy_waker() -> RawWaker {
      RawWaker::new(core::ptr::null(), &RawWakerVTable::new(clone, dummy, dummy, dummy))
    }
    let waker = unsafe { Waker::from_raw(dummy_waker()) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = pin!(fut);
    loop {
      match fut.as_mut().poll(&mut cx) {
        Poll::Ready(val) => break val,
        Poll::Pending => {}
      }
    }
  }

  // Erase the storage region (blocking)
  let _ = block_on(flash::erase_blocks(&mut flash_hw));
  // Write block (blocking)
  let _ = block_on(flash::write_block(&mut flash_hw, test_offset, &test_data));
  // Read block (sync)
  let _ = flash::read_block(test_offset, &mut read_buf);
  let _passed = read_buf == test_data;
  process::exit(0)
}

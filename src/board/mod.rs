//! Board selection driven by Cargo features. Exactly one MCU feature must
//! be enabled (`stm32f413` or `stm32f446`). The chosen module is re-exported
//! as `BoardConfig` so the rest of the crate stays board-agnostic.

mod base;
pub use base::{Board, BoardHardware};

#[cfg(feature = "stm32f413")]
mod nucleo144_f413zh;
#[cfg(feature = "stm32f413")]
pub use nucleo144_f413zh::BoardConfig;

#[cfg(feature = "stm32f446")]
mod nucleo_f446re;
#[cfg(feature = "stm32f446")]
pub use nucleo_f446re::BoardConfig;

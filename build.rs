//! Build script: writes the correct `memory.x` based on the selected MCU
//! feature, so the linker (link.x) picks it up via OUT_DIR. This replaces
//! the previous `setup.sh` shell-driven configuration.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
  let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

  let (memory_x, name): (&str, &str) = match (cfg!(feature = "stm32f413"), cfg!(feature = "stm32f446")) {
    (true, false) => (include_str!("memory/stm32f413zh.x"), "stm32f413zh"),
    (false, true) => (include_str!("memory/stm32f446re.x"), "stm32f446re"),
    (true, true) => panic!("Enable only one MCU feature (stm32f413 OR stm32f446)"),
    (false, false) => panic!("Enable one MCU feature: --features stm32f413 OR --features stm32f446"),
  };

  fs::write(out.join("memory.x"), memory_x).expect("failed to write memory.x");

  println!("cargo:rustc-link-search={}", out.display());
  println!("cargo:rustc-env=BOARD_MEMORY_X={}", name);
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=memory/stm32f413zh.x");
  println!("cargo:rerun-if-changed=memory/stm32f446re.x");
}

pub mod emulator;
pub use emulator::{input::Input, machine::Machine};

//
// benchmarks
// #[cfg(feature = "bench")]
// pub mod bench;

//
// test_runner
#[cfg(feature = "test_runner")]
pub mod test_runner;
#[cfg(feature = "test_runner")]
pub use test_runner::*;

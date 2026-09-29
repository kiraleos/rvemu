//! An emulator for the RV32I base integer instruction set.
//!
//! [`cpu::Cpu`] is the interpreter: it holds the registers, the program counter
//! and memory, and knows how to decode, execute and run instructions.
//! [`cpu::RunConfig`] and [`cpu::Outcome`] are how a caller asks for a run and
//! reads back how it ended, and [`cpu::LoadError`] is why an image could not be
//! loaded. Nothing here depends on the command line, so the emulator can be
//! driven from a test or embedded in something else.
//!
//! ```
//! use rvemu::emulator::cpu::{Cpu, Outcome, RunConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut cpu = Cpu::new(16);
//! cpu.load("tests/simple")?;
//! assert_eq!(cpu.run(&RunConfig::default()), Outcome::Exited(0));
//! # Ok(())
//! # }
//! ```
pub mod emulator;

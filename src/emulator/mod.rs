//! The emulator itself: instruction decoding on one side, the CPU that executes
//! decoded instructions on the other.
//!
//! [`instruction`] is a pure function from a 32-bit word to the fields its
//! format defines. [`cpu`] holds the state that executing them changes, and the
//! run loop that drives it. [`disassembly`] turns a decoded instruction back
//! into the text of an instruction, which the run loop prints as it goes and
//! which `--disassemble` prints for a program that is not running.

pub mod cpu;
pub mod disassembly;
pub mod instruction;

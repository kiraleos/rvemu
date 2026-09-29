//! Runs the pre-built RV32I test programs in `tests/`.
//!
//! They come from [riscv-tests], and each one exercises a single instruction
//! over a range of operands, exiting with the number of the test that failed or
//! with zero when the instruction behaves as the specification says. A non-zero
//! status therefore names the instruction that went wrong.
//!
//! [riscv-tests]: https://github.com/riscv/riscv-tests

use rvemu::emulator::cpu::{Cpu, Outcome, RunConfig};
use std::path::PathBuf;

/// The memory each program needs, in kibibytes.
const MEMORY_KIB: usize = 16;

/// One program per RV32I instruction, plus `simple`, which is the smallest
/// complete program here.
const PROGRAMS: &[&str] = &[
    "add", "addi", "and", "andi", "auipc", "beq", "bge", "bgeu", "blt", "bltu", "bne", "fence_i",
    "jal", "jalr", "lb", "lbu", "lh", "lhu", "lui", "lw", "or", "ori", "sb", "sh", "simple", "sll",
    "slli", "slt", "slti", "sltiu", "sltu", "sra", "srai", "srl", "srli", "sub", "sw", "xor",
    "xori",
];

fn program(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name)
}

#[test]
fn every_test_program_exits_successfully() {
    for name in PROGRAMS {
        let mut cpu = Cpu::new(MEMORY_KIB);
        cpu.load(program(name)).expect("test program is missing");
        assert_eq!(
            cpu.run(&RunConfig::default()),
            Outcome::Exited(0),
            "{name} did not pass"
        );
    }
}

#[test]
fn every_test_program_is_listed() {
    // Catches a riscv-tests program being added without a row above. The names
    // are compared as a sorted list, so the failure says which one is missing.
    let mut found: Vec<String> = std::fs::read_dir(program(""))
        .expect("the tests directory is missing")
        .map(|entry| entry.expect("a directory entry").file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.contains('.')) // skips the .dump files, and this one
        .collect();
    found.sort();
    assert_eq!(found, PROGRAMS);
}

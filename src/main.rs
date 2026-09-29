mod emulator;

#[cfg(test)]
mod tests;

use clap::Parser;
use emulator::cpu::{Cpu, Outcome, RunConfig};
use std::path::PathBuf;
use std::process::ExitCode;

///  A RISC-V emulator, specifically the RV32I base integer instruction set.
#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The path of the file to be executed
    #[clap(parse(from_os_str), value_name = "FILE")]
    file: PathBuf,

    /// Print instructions as they are executed
    #[clap(short, long)]
    pub debug: bool,

    /// Show register values after each instruction
    #[clap(short, long)]
    pub registers: bool,

    /// Show register ABI names or numeric values (x0-x31)
    /// Use with the `--registers` option.
    #[clap(short, long)]
    pub aliases: bool,

    /// Interactive mode. Use with either `--registers` and/or `--debug`
    #[clap(short, long)]
    pub interactive: bool,

    /// Override ELF entry point
    #[clap(long, value_name = "address")]
    pub pc: Option<String>,

    /// Provide a stack of "infinite" size.
    /// This sets the stack pointer before execution, so it might cause undefined behaviour.
    #[clap(short, long)]
    pub stack: bool,

    /// Set memory size in KiB (default = 16)
    #[clap(long, value_name = "size")]
    pub mem: Option<String>,
}

/// The default size of emulated memory, in kibibytes.
const DEFAULT_MEMORY_KIB: usize = 16;

fn main() -> ExitCode {
    let args = Args::parse();

    let mem = args
        .mem
        .as_deref()
        .and_then(|size| size.parse().ok())
        .unwrap_or(DEFAULT_MEMORY_KIB);
    let mut cpu = Cpu::new(mem);
    if let Err(err) = cpu.load(&args.file) {
        eprintln!("{}: {err}", args.file.display());
        return ExitCode::FAILURE;
    }

    let config = RunConfig {
        debug: args.debug,
        registers: args.registers,
        aliases: args.aliases,
        interactive: args.interactive,
        pc: args
            .pc
            .as_deref()
            .and_then(|pc| u32::from_str_radix(pc, 16).ok()),
        stack: args.stack,
    };

    match cpu.run(&config) {
        Outcome::Exited(code) => ExitCode::from(code as u8),
        _ => ExitCode::FAILURE,
    }
}

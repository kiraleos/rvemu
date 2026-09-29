use clap::Parser;
use rvemu::emulator::cpu::{Cpu, Outcome, RunConfig};
use std::path::PathBuf;
use std::process::ExitCode;

/// The default size of emulated memory, in kibibytes.
const DEFAULT_MEMORY_KIB: usize = 16;

///  A RISC-V emulator, specifically the RV32I base integer instruction set.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path of the file to be executed
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Print instructions as they are executed
    #[arg(short, long)]
    debug: bool,

    /// Show register values after each instruction
    #[arg(short, long)]
    registers: bool,

    /// Show register ABI names or numeric values (x0-x31)
    /// Use with the `--registers` option.
    #[arg(short, long)]
    aliases: bool,

    /// Interactive mode. Use with either `--registers` and/or `--debug`
    #[arg(short, long)]
    interactive: bool,

    /// Override ELF entry point, in hexadecimal
    #[arg(long, value_name = "address", value_parser = parse_hex)]
    pc: Option<u32>,

    /// Provide a stack of "infinite" size.
    /// This sets the stack pointer before execution, so it might cause undefined behaviour.
    #[arg(short, long)]
    stack: bool,

    /// Set memory size in KiB
    #[arg(long, value_name = "size", default_value_t = DEFAULT_MEMORY_KIB)]
    mem: usize,
}

/// Parses a bare hexadecimal number, which is how `--pc` and `mem` are written.
fn parse_hex(text: &str) -> Result<u32, String> {
    u32::from_str_radix(text.strip_prefix("0x").unwrap_or(text), 16).map_err(|err| err.to_string())
}

fn main() -> ExitCode {
    let args = Args::parse();

    let mut cpu = Cpu::new(args.mem);
    if let Err(err) = cpu.load(&args.file) {
        eprintln!("{}: {err}", args.file.display());
        return ExitCode::FAILURE;
    }

    let config = RunConfig {
        debug: args.debug,
        registers: args.registers,
        aliases: args.aliases,
        interactive: args.interactive,
        pc: args.pc,
        stack: args.stack,
    };

    match cpu.run(&config) {
        Outcome::Exited(code) => ExitCode::from(code as u8),
        _ => ExitCode::FAILURE,
    }
}

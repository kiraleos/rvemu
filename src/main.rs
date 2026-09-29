mod emulator;
mod tests;
use clap::Parser;
use emulator::cpu::Cpu;

/// The default amount of memory, in KiB.
const DEFAULT_MEM_KIB: usize = 16;

///  A RISC-V emulator, specifically the RV32I base integer instruction set.
#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The path of the file to be executed
    #[clap(parse(from_os_str), value_name = "FILE")]
    file: std::path::PathBuf,

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

    /// Override ELF entry point (hexadecimal, e.g. 1000 or 0x1000)
    #[clap(long, value_name = "address", parse(try_from_str = parse_hex))]
    pub pc: Option<u32>,

    /// Provide a stack of "infinite" size.
    /// This sets the stack pointer before execution, so it might cause undefined behaviour.
    #[clap(short, long)]
    pub stack: bool,

    /// Set memory size in KiB (default = 16)
    #[clap(long, value_name = "size")]
    pub mem: Option<String>,
}

/// Parse a hexadecimal address, rejecting anything that is not a valid u32.
fn parse_hex(s: &str) -> Result<u32, String> {
    let trimmed = s.trim();
    let digits = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u32::from_str_radix(digits, 16)
        .map_err(|e| format!("'{}' is not a 32-bit hex address: {}", s, e))
}

/// The requested memory size in KiB, or the default if it was not given.
fn mem_kib(args: &Args) -> Result<usize, String> {
    match &args.mem {
        None => Ok(DEFAULT_MEM_KIB),
        Some(raw) => raw
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("--mem '{}' is not a size in KiB: {}", raw, e))
            .and_then(|kib| {
                if kib == 0 {
                    return Err(String::from(
                        "--mem must be greater than zero",
                    ));
                }
                // `mem_size * 1024` is done in usize and would wrap for
                // absurd inputs, producing a tiny allocation.
                kib.checked_mul(1024).ok_or_else(|| {
                    format!("--mem {} KiB is too large", kib)
                })
            })
            .map(|bytes| bytes / 1024),
    }
}

fn main() {
    let args = Args::parse();

    let kib = mem_kib(&args).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(2);
    });
    let mut cpu = Cpu::new(kib).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(2);
    });
    cpu.load(
        args.file
            .clone()
            .into_os_string()
            .to_str()
            .expect("not valid unicode"),
    )
    .unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    });

    let code = cpu.run(args);
    // Negative codes are emulator-level failures (bad image, trap, PC
    // overflow). Map them onto a nonzero exit status so a script can tell
    // "the guest asked for this" apart from "the guest could not run".
    if code < 0 {
        std::process::exit(1);
    }
    std::process::exit(code);
}

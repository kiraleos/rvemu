mod emulator;
mod tests;
use clap::Parser;
use emulator::cpu::Cpu;

///  A RISC-V emulator, specifically the RV32I base integer instruction set.
#[derive(Parser)]
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

    /// Override ELF entry point
    #[clap(long, value_name = "address")]
    pub pc: Option<String>,

    /// Provide a stack of "infinite" size.
    /// This sets the stack pointer before execution, so it might cause undefined behaviour.
    #[clap(short, long)]
    pub stack: bool,

    /// Set memory size in KiB (default = 16KiB)
    #[clap(long, value_name = "size")]
    pub mem: Option<String>,
}
fn main() {
    let args = Args::parse();

    let mem = args.mem.clone();
    let mut cpu = match mem {
        Some(mem) => Cpu::new(str::parse(&*mem).unwrap_or(16)),
        None => Cpu::new(16),
    };
    cpu.load(
        args.file
            .clone()
            .into_os_string()
            .to_str()
            .expect("not valid unicode"),
    );

    cpu.run(args);

    // TODO
    // add commands to interactive mode
    //      1. `mem <addr|range>` to show a memory location
    //      2. `reg <x> to show a single register's value`
    //      3. `regset <reg> <value>` to set a register's value
    //      4. `memset <addr> <value>` to set a memory location's value
}

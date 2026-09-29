// The emulator reinterprets bit patterns constantly: `slt` compares registers as
// signed, a branch target is a signed offset added to a program counter, and a
// sign extended immediate is a `u32` that has to print as the negative number it
// is. Those are the semantics of the instruction set rather than accidents, so
// the lints about them are muted here instead of at each of the sites.
#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]

use super::disassembly::Disassembly;
use super::instruction::{Instruction, SHIFT_AMOUNT_MASK, sign_extend};
use elf_rs::{Elf, ElfFile, ElfMachine};
use std::error;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::ops::Range;
use std::path::Path;

/// The ABI name of every register, indexed by register number.
const ALIASES: [&str; REGISTER_COUNT] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

/// Number of general purpose registers, `x0` through `x31`.
const REGISTER_COUNT: usize = 32;

/// How many registers `print_registers` puts on one line.
const REGISTERS_PER_LINE: usize = 4;

/// `x2`, the stack pointer.
const SP: usize = 2;
/// `x10`, the first argument register, and where `exit` reads its status.
const A0: usize = 10;
/// `x11` and `x12`, the second and third arguments of a system call.
const A1: usize = 11;
const A2: usize = 12;
/// `x17`, the register holding the system call number.
const A7: usize = 17;

/// The system calls this emulator implements, by their Linux numbers.
const WRITE_SYSCALL: u32 = 64;
const EXIT_SYSCALL: u32 = 93;

/// What a system call leaves in `a0` when it cannot do what was asked. Any
/// negative value means failure and a program only has to test the sign; this
/// is -1 rather than a Linux errno, which would imply an ABI that is not
/// otherwise implemented here.
const SYSCALL_ERROR: u32 = u32::MAX;

/// Every RV32I instruction is four bytes wide, which is also the width of an
/// address on RV32I.
const INSTRUCTION_SIZE: u32 = 4;

/// Memory is sized in kibibytes on the command line.
const KIB: usize = 1024;

/// How a program should be run.
///
/// This is deliberately separate from the command line: the CPU has no business
/// knowing about flags, and in particular about the path of the file it was
/// loaded from.
#[derive(Debug, Default, Clone, Copy)]
pub struct RunConfig {
    /// Print every instruction as it is executed.
    pub debug: bool,
    /// Print the register file alongside each instruction.
    pub registers: bool,
    /// Name registers `sp` and `a0` rather than `x2` and `x10`.
    pub aliases: bool,
    /// Run one instruction per line of input instead of running freely.
    pub interactive: bool,
    /// Start here rather than at the ELF entry point.
    pub pc: Option<u32>,
    /// Point the stack pointer at the top of memory before starting.
    pub stack: bool,
}

/// Why the emulator stopped running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The program called `exit` with this status.
    Exited(i32),
    /// The program counter ran past the end of memory.
    PcOverflow,
    /// The program made a system call the emulator does not implement. The
    /// payload is the system call number, which lives in `a7`.
    UnsupportedSyscall(u32),
    /// The program executed `ebreak`.
    Breakpoint,
    /// The program reached an instruction the emulator does not implement.
    UnsupportedInstruction,
}

/// Why loading an ELF image into memory failed.
#[derive(Debug)]
pub enum LoadError {
    /// The file could not be opened or read.
    Io(io::Error),
    /// The file is not a well formed ELF image.
    Malformed,
    /// The image is not a RISC-V executable.
    WrongArchitecture,
    /// The image does not fit in the configured amount of memory.
    ImageTooLarge {
        /// The size of the image, in bytes.
        size: usize,
        /// How much memory the CPU has, in bytes.
        capacity: usize,
    },
    /// The entry point is not inside a loadable segment, or does not fit in
    /// the 32-bit address space.
    EntryPointNotLoadable,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(err) => write!(f, "{err}"),
            LoadError::Malformed => write!(f, "not a valid ELF file"),
            LoadError::WrongArchitecture => write!(f, "not a RISC-V executable"),
            LoadError::ImageTooLarge { size, capacity } => write!(
                f,
                "the image needs {size} bytes of memory but there are {capacity}"
            ),
            LoadError::EntryPointNotLoadable => {
                write!(f, "the entry point is not inside a loadable segment")
            }
        }
    }
}

impl error::Error for LoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            LoadError::Io(err) => Some(err),
            _ => None,
        }
    }
}

/// Fills `line` with the trace line for one instruction: the address it is at,
/// the word it encoded as, and what it means.
///
/// The trace and the listing both go through here, so a line of a trace is a
/// line of a listing and the two cannot drift apart.
fn trace_line(line: &mut String, pc: u32, word: u32, disasm: &impl fmt::Display) {
    let _ = write!(line, "{pc:<08x}:   {word:08x}          \t{disasm}");
}

/// Where the disassembly of an instruction goes.
///
/// Only `--debug` and interactive mode ever read it, and formatting it is most
/// of the cost of executing a simple instruction, so a run nobody is watching
/// writes nothing at all. The arguments are still assembled at each call site,
/// because that is what a format string is for; only the formatting is skipped.
enum Trace<'a> {
    /// Nobody is looking, so there is nothing to write.
    Off,
    /// A disassembly is being collected.
    On(&'a mut String),
}

impl Trace<'_> {
    /// Discards whatever the previous instruction wrote.
    fn clear(&mut self) {
        if let Self::On(text) = self {
            text.clear();
        }
    }

    /// Records the disassembly of one instruction.
    fn write(&mut self, args: fmt::Arguments) {
        if let Self::On(text) = self {
            let _ = text.write_fmt(args);
        }
    }
}

/// A RISC-V RV32I CPU, as described by the unprivileged base integer spec.
pub struct Cpu {
    memory: Vec<u8>,
    registers: [u32; REGISTER_COUNT],
    pc: u32,
    /// The addresses the loaded image's code occupies, which is what
    /// [`Cpu::disassembly`] lists. Empty until an image is loaded. These are the
    /// addresses the emulator gave the bytes rather than the virtual addresses
    /// the ELF calls them, for the same reason the program counter is: see
    /// [`Cpu::load`].
    code: Range<u32>,
}

/// What executing a single instruction did, as far as the run loop cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Execution carries on at the following instruction.
    Next,
    /// Control flow moved elsewhere; `pc` already holds the target.
    Jump,
    /// `ecall`, so the run loop has to service a system call.
    Ecall,
    /// `ebreak`, a requested trap the run loop has to report.
    Breakpoint,
    /// An instruction this emulator does not implement, so it should stop.
    Unsupported,
}

/// Parses the hex address that `mem` was given, into a message fit to print.
fn parse_hex_address(argument: Option<&str>) -> Result<u32, String> {
    let Some(text) = argument else {
        return Err("expected a hex address".to_owned());
    };
    u32::from_str_radix(text, 16).map_err(|err| err.to_string())
}

impl Cpu {
    /// Creates a CPU with `mem_size_kib` kibibytes of memory and every register
    /// zeroed.
    ///
    /// # Panics
    ///
    /// If the size is zero, or too large for a `usize` to hold in bytes.
    ///
    /// # Panics
    ///
    /// If the size is zero, or too large to address.
    pub fn new(mem_size_kib: usize) -> Self {
        let size = mem_size_kib
            .checked_mul(KIB)
            .filter(|size| *size != 0)
            .expect("memory size must be at least one kibibyte and must fit in a usize");
        Cpu {
            memory: vec![0; size],
            registers: [0; REGISTER_COUNT],
            pc: 0,
            code: 0..0,
        }
    }

    /// Loads an ELF image into memory and points the program counter at its
    /// entry point.
    ///
    /// The image is copied in whole, at the addresses its file offsets give it,
    /// instead of being unpacked segment by segment. Those are the same thing
    /// only when every segment's file offset matches its virtual address, which
    /// holds for the programs this emulator is meant to run but not in general:
    /// a segment at `p_offset != p_vaddr` ends up somewhere else, and so do
    /// anything but the file's own bytes, which are not in memory at all.
    ///
    /// The segment holding the entry point is remembered as the program's code,
    /// which is what [`Cpu::disassembly`] lists.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] if the file cannot be read, is not a RISC-V ELF
    /// image, does not fit in the configured memory, or has an entry point that
    /// is not inside a loadable segment.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), LoadError> {
        let image = fs::read(path).map_err(LoadError::Io)?;
        let elf = Elf::from_bytes(&image).map_err(|_| LoadError::Malformed)?;
        if elf.elf_header().machine() != ElfMachine::RISC_V {
            return Err(LoadError::WrongArchitecture);
        }
        if image.len() > self.memory.len() {
            return Err(LoadError::ImageTooLarge {
                size: image.len(),
                capacity: self.memory.len(),
            });
        }

        // The entry point is a virtual address while the program counter reads
        // memory, so it has to be translated through the segment holding it.
        let entry = elf.entry_point();
        let segment = elf
            .program_header_iter()
            .find(|phdr| phdr.vaddr() <= entry && entry < phdr.vaddr() + phdr.memsz())
            .ok_or(LoadError::EntryPointNotLoadable)?;
        let pc = u32::try_from(entry - segment.vaddr() + segment.offset())
            .map_err(|_| LoadError::EntryPointNotLoadable)?;

        // The code is the part of that segment which is in the file. The rest of
        // it is memory a loader would zero fill, which is not instructions, and
        // a segment claiming more bytes than the file holds is malformed and has
        // nothing behind it to read.
        let start = u32::try_from(segment.offset()).unwrap_or(0);
        let end = u32::try_from(segment.offset() + segment.filesz()).unwrap_or(u32::MAX);
        let image_end = u32::try_from(image.len()).unwrap_or(u32::MAX);
        self.code = start..end.min(image_end);

        self.memory[..image.len()].copy_from_slice(&image);
        self.pc = pc;
        Ok(())
    }

    /// The program's code as a disassembly: one line per instruction, from the
    /// first to the last word of the segment the entry point is in.
    ///
    /// Each line is the one `--debug` prints for that instruction while a run is
    /// executing it, so a listing and a trace of the same program can be read
    /// against each other. Nothing is executed here, which is the point: a
    /// listing covers the code a run never reaches, such as the setup the entry
    /// point jumps over, and the words a run would stop on are named as they
    /// are passed over.
    ///
    /// The addresses are the ones the emulator itself uses, which for an image
    /// loaded the way [`Cpu::load`] loads it are its file offsets rather than the
    /// virtual addresses the ELF calls them.
    pub fn disassembly(&self) -> impl Iterator<Item = String> + '_ {
        let cpu = self;
        self.code
            .clone()
            .step_by(INSTRUCTION_SIZE as usize)
            .map(move |pc| {
                let word = cpu.load_bytes::<4>(pc);
                let mut line = String::new();
                trace_line(
                    &mut line,
                    pc,
                    word,
                    &Disassembly::new(pc, &Instruction::decode(word)),
                );
                line
            })
    }

    /// Prints the program counter and every register, four to a line.
    ///
    /// `aliases` selects between the ABI names (`sp`, `a0`, ...) and the
    /// numeric ones (`x2`, `x10`, ...).
    pub fn print_registers(&self, aliases: bool) {
        println!(" pc: 0x{:08x}", self.pc);
        let mut line = String::new();
        for (index, &value) in self.registers.iter().enumerate() {
            if aliases {
                let _ = write!(line, "{:>4}: 0x{value:08x}  ", ALIASES[index]);
            } else {
                // "x9" is a character shorter than "x10", so the low registers
                // get a space to keep the columns lined up.
                if index < 10 {
                    line.push(' ');
                }
                let _ = write!(line, "x{index}: 0x{value:08x}    ");
            }
            if (index + 1) % REGISTERS_PER_LINE == 0 {
                line.push('\n');
            }
        }
        println!("{line}");
    }

    /// Reads the `N` little-endian bytes at `address`, zero extended.
    ///
    /// An access outside memory panics, which is how this emulator reports a
    /// bad address: it implements neither traps nor page faults.
    fn load_bytes<const N: usize>(&self, address: u32) -> u32 {
        let start = address as usize;
        let mut bytes = [0u8; 4];
        bytes[..N].copy_from_slice(&self.memory[start..start + N]);
        u32::from_le_bytes(bytes)
    }

    /// Writes the `N` least significant bytes of `value` at `address`.
    fn store_bytes<const N: usize>(&mut self, address: u32, value: u32) {
        let start = address as usize;
        self.memory[start..start + N].copy_from_slice(&value.to_le_bytes()[..N]);
    }

    /// Reads the instruction word that the program counter points at.
    fn fetch(&self) -> u32 {
        self.load_bytes::<{ INSTRUCTION_SIZE as usize }>(self.pc)
    }

    /// Executes `inst`, updating the registers and the program counter.
    ///
    /// The disassembly that `--debug` prints is written to `trace`. It is
    /// produced here, from the instruction alone, rather than being spelled out
    /// again in each arm: what an instruction is called is the disassembler's
    /// business, and a mnemonic that this function and the disassembler each
    /// worked out for themselves is a mnemonic that can disagree with the
    /// instruction it names.
    #[allow(clippy::too_many_lines)]
    fn execute(&mut self, inst: &Instruction, trace: &mut Trace<'_>) -> Step {
        trace.clear();
        // Before the arms below, because they move the program counter and the
        // offsets of a branch and a jump are counted from the instruction
        // itself.
        let disassembly = Disassembly::new(self.pc, inst);
        trace.write(format_args!("{disassembly}"));

        // The arms below only have to deal with instructions that redirect
        // control flow; the shared epilogue advances the program counter and
        // re-clamps x0 for everything else.
        let step = match *inst {
            Instruction::R {
                rd,
                funct3,
                rs1,
                rs2,
                funct7,
            } => {
                let (a, b) = (self.registers[rs1], self.registers[rs2]);
                let value = match funct3 {
                    // `add` and `sub` share funct3 and are told apart by funct7.
                    0b000 => match funct7 {
                        0b000_0000 => a.wrapping_add(b),
                        0b010_0000 => a.wrapping_sub(b),
                        other => panic!("unknown R funct7: {other:#09b}"),
                    },
                    0b001 => a << (b & SHIFT_AMOUNT_MASK),
                    0b010 => u32::from((a as i32) < (b as i32)),
                    0b011 => u32::from(a < b),
                    0b100 => a ^ b,
                    // `srl` and `sra`, like `add` and `sub`, share funct3.
                    0b101 => match funct7 {
                        0b000_0000 => a >> (b & SHIFT_AMOUNT_MASK),
                        0b010_0000 => ((a as i32) >> (b & SHIFT_AMOUNT_MASK)) as u32,
                        other => panic!("unknown R funct7: {other:#09b}"),
                    },
                    0b110 => a | b,
                    0b111 => a & b,
                    other => {
                        panic!("execute: unimplemented R funct3: {other:#05b}")
                    }
                };
                self.registers[rd] = value;
                Step::Next
            }
            Instruction::OpImm {
                rd,
                funct3,
                rs1,
                imm,
            } => {
                let a = self.registers[rs1];
                match funct3 {
                    // The shifts are the only OP-IMM instructions that check the
                    // funct7 half of their immediate.
                    0b001 | 0b101 => {
                        let shamt = imm & SHIFT_AMOUNT_MASK;
                        let value = if funct3 == 0b001 {
                            a << shamt
                        } else {
                            match imm >> 5 & 0b111_1111 {
                                0b000_0000 => a >> shamt,
                                0b010_0000 => {
                                    // Shifting right and then refilling the
                                    // vacated high bits with copies of the
                                    // sign bit is an arithmetic shift.
                                    sign_extend(a >> shamt, 32 - shamt)
                                }
                                other => panic!("unknown shift funct7: {other:#09b}"),
                            }
                        };
                        self.registers[rd] = value;
                    }
                    _ => {
                        let value = match funct3 {
                            0b000 => a.wrapping_add(imm),
                            0b010 => u32::from((a as i32) < imm as i32),
                            0b011 => u32::from(a < imm),
                            0b100 => a ^ imm,
                            0b110 => a | imm,
                            0b111 => a & imm,
                            other => panic!("unknown I funct3: {other:#05b}"),
                        };
                        self.registers[rd] = value;
                    }
                }
                Step::Next
            }
            Instruction::Load {
                rd,
                funct3,
                rs1,
                imm,
            } => {
                let address = self.registers[rs1].wrapping_add(imm);
                let value = match funct3 {
                    0b000 => sign_extend(self.load_bytes::<1>(address), 8),
                    0b001 => sign_extend(self.load_bytes::<2>(address), 16),
                    0b010 => self.load_bytes::<4>(address),
                    0b100 => self.load_bytes::<1>(address),
                    0b101 => self.load_bytes::<2>(address),
                    other => panic!("unknown I funct3: {other:#05b}"),
                };
                self.registers[rd] = value;
                Step::Next
            }
            Instruction::Store {
                rs1,
                rs2,
                imm,
                funct3,
            } => {
                let address = self.registers[rs1].wrapping_add(imm);
                let value = self.registers[rs2];
                match funct3 {
                    0b000 => self.store_bytes::<1>(address, value),
                    0b001 => self.store_bytes::<2>(address, value),
                    0b010 => self.store_bytes::<4>(address, value),
                    other => panic!("unknown S funct3: {other:#05b}"),
                }
                Step::Next
            }
            Instruction::Branch {
                rs1,
                rs2,
                imm,
                funct3,
            } => {
                let target = (self.pc as i32).wrapping_add(imm as i32);
                let (a, b) = (self.registers[rs1], self.registers[rs2]);
                let taken = match funct3 {
                    0b000 => a == b,
                    0b001 => a != b,
                    0b100 => (a as i32) < (b as i32),
                    0b101 => (a as i32) >= (b as i32),
                    0b110 => a < b,
                    0b111 => a >= b,
                    other => {
                        panic!("execute: unimplemented B funct3: {other:#05b}")
                    }
                };
                if taken {
                    self.pc = target as u32;
                    Step::Jump
                } else {
                    Step::Next
                }
            }
            Instruction::Jump { rd, imm } => {
                self.registers[rd] = self.pc.wrapping_add(4);
                self.pc = self.pc.wrapping_add(imm);
                Step::Jump
            }
            Instruction::JumpRegister { rd, rs1, imm } => {
                // Read the base before writing rd: `jalr x1, x1, 0` is legal.
                // The spec requires the low bit of the target to be zero.
                let target = self.registers[rs1].wrapping_add(imm) & !1;
                self.registers[rd] = self.pc.wrapping_add(4);
                self.pc = target;
                Step::Jump
            }
            Instruction::Lui { rd, imm } => {
                self.registers[rd] = imm << 12;
                Step::Next
            }
            Instruction::Auipc { rd, imm } => {
                self.registers[rd] = self.pc.wrapping_add(imm << 12);
                Step::Next
            }
            Instruction::System { imm, funct3, .. } => match funct3 {
                // `ecall`, `ebreak` and `mret` share funct3 and are told apart
                // by the immediate, which for them is a plain 12-bit field
                // rather than a sign extended one.
                0b000 => match imm {
                    0x000 => Step::Ecall,
                    // Both ECALL and EBREAK "cause a precise requested trap to
                    // the supporting execution environment" (RV32I 1.9), and this
                    // emulator has no traps, so it reports the breakpoint and
                    // stops.
                    0x001 => Step::Breakpoint,
                    // MRET belongs to the privileged architecture, which this
                    // emulator does not implement. It is left as a no-op because
                    // the riscv-tests programs use it in their machine-mode setup
                    // and would never reach the instruction they are testing.
                    0b0011_0000_0010 => Step::Next,
                    other => panic!("unknown I imm: {other:#014b}"),
                },
                // This emulator implements no CSR file, and the same is true of
                // the riscv-tests programs' machine-mode setup: they write
                // mtvec, satp and pmpcfg0 before the test body. Discarding
                // every access is what lets them run to completion, so a
                // program that relies on a CSR value reading one back gets
                // zero instead.
                0b001 | 0b010 | 0b011 | 0b101 | 0b110 | 0b111 => Step::Next,
                other => panic!("unknown I funct3: {other:#05b}"),
            },
            Instruction::Fence { .. } => Step::Next,
            Instruction::Unsupported => Step::Unsupported,
        };

        self.registers[0] = 0;
        if step != Step::Jump {
            self.pc = self.pc.wrapping_add(INSTRUCTION_SIZE);
        }
        step
    }

    /// Services the `ecall` that was just retired, returning the outcome if it
    /// ended the run.
    fn handle_ecall(&mut self, debug: bool) -> Option<Outcome> {
        let number = self.registers[A7];
        match number {
            EXIT_SYSCALL => {
                let code = self.registers[A0] as i32;
                println!("Program exited with exit code: {code}");
                Some(Outcome::Exited(code))
            }
            // `write(fd, buf, count)`. There is no input, and stderr is not kept
            // apart from stdout, so every descriptor writes to stdout.
            WRITE_SYSCALL => {
                let count = self.registers[A2];
                self.registers[A0] = match self.guest_bytes(self.registers[A1], count) {
                    Some(bytes) => {
                        // The program is about to read its own return value, so
                        // its output has to be in place before it runs again.
                        let mut stdout = io::stdout().lock();
                        stdout.write_all(bytes).expect("stdout is closed");
                        stdout.flush().expect("stdout is closed");
                        count
                    }
                    // A bad buffer is reported to the program rather than
                    // aborting the run, which is what a bad load does: a system
                    // call has a return value to fail in.
                    None => SYSCALL_ERROR,
                };
                None
            }
            _ => {
                if debug {
                    println!("Unimplemented ECALL: {number}");
                }
                Some(Outcome::UnsupportedSyscall(number))
            }
        }
    }

    /// The `count` bytes of memory at `address`, or `None` if that is not all
    /// inside memory.
    fn guest_bytes(&self, address: u32, count: u32) -> Option<&[u8]> {
        let start = address as usize;
        let end = start.checked_add(count as usize)?;
        self.memory.get(start..end)
    }

    /// Interprets one line of interactive mode input, returning the reply to
    /// print back, if there is one.
    ///
    /// An empty line means "step the program" and has no reply; the run loop
    /// tells that case apart before calling.
    fn command(&self, input: &str) -> Option<String> {
        let mut words = input.split_whitespace();
        match words.next()? {
            "mem" => Some(self.read_word(words.next())),
            "reg" => Some(self.read_register(words.next())),
            command => Some(format!("Unknown command: {command}")),
        }
    }

    /// The 32-bit little-endian word at the hex address in `argument`.
    fn read_word(&self, argument: Option<&str>) -> String {
        let address = match parse_hex_address(argument) {
            Ok(address) => address,
            Err(reason) => return format!("bad argument: {reason}"),
        };
        // The last byte of a four byte word, without overflowing on a huge
        // address.
        match address.checked_add(3) {
            Some(end) if (end as usize) < self.memory.len() => {
                format!("{:#010x}", self.load_bytes::<4>(address))
            }
            _ => "bad argument: memory out of bounds".to_owned(),
        }
    }

    /// The value of the register named in `argument`.
    fn read_register(&self, argument: Option<&str>) -> String {
        let index = match argument {
            Some(number) => match number.parse::<usize>() {
                Ok(index) => index,
                Err(err) => return format!("bad argument: {err}"),
            },
            None => return "bad argument: expected a register number".to_owned(),
        };
        match self.registers.get(index) {
            Some(&value) => format!("{value:#x}"),
            None => "bad argument: no such register".to_owned(),
        }
    }

    /// Runs the program until it cannot go any further.
    ///
    /// # Panics
    ///
    /// If the program reads or writes outside the configured memory. The
    /// emulator implements no traps or page faults, so a bad address is a bug in
    /// the guest program and is reported as one.
    pub fn run(&mut self, config: &RunConfig) -> Outcome {
        if let Some(pc) = config.pc {
            self.pc = pc;
        }
        if config.stack {
            // The highest address in memory, so that a stack grows downwards
            // into it. RV32I addresses are 32 bits, so more memory than that is
            // memory a program cannot name.
            self.registers[SP] = u32::try_from(self.memory.len() - 1)
                .expect("memory is larger than the address space");
        }

        // All three are reused for the life of the run, so that neither the
        // prompt, nor the disassembly, nor the line they are printed on
        // allocates per instruction.
        let mut line = String::new();
        let mut disasm = String::new();
        let mut traced = String::new();
        // Tracing is the point of stepping by hand, so interactive mode traces
        // whether or not `--debug` was asked for.
        let tracing = config.debug || config.interactive;
        loop {
            // Interactive mode asks before every instruction. With no one to
            // interrupt, the state is printed before each instruction instead
            // of after it.
            let stepped = if config.interactive {
                line.clear();
                print!("> ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut line).unwrap();
                line.pop();
                if let Some(reply) = self.command(&line) {
                    println!("{reply}");
                }
                // An empty line steps the program; anything else was a command
                // that has already been handled.
                line.is_empty()
            } else {
                if config.registers {
                    self.print_registers(config.aliases);
                }
                true
            };

            let pc = self.pc;
            let word = self.fetch();
            let instruction = Instruction::decode(word);

            let mut step = None;
            if stepped {
                let mut trace = if tracing {
                    Trace::On(&mut disasm)
                } else {
                    Trace::Off
                };
                step = Some(self.execute(&instruction, &mut trace));
                if config.interactive && config.registers {
                    self.print_registers(config.aliases);
                }
                if tracing {
                    trace_line(&mut traced, pc, word, &disasm);
                    println!("{traced}");
                }
            }

            if (self.pc as usize) >= self.memory.len() {
                if config.debug {
                    println!("PC overflow.");
                }
                return Outcome::PcOverflow;
            }
            match step {
                Some(Step::Ecall) => {
                    if let Some(outcome) = self.handle_ecall(config.debug) {
                        return outcome;
                    }
                }
                Some(Step::Breakpoint) => {
                    println!("Program hit a breakpoint at 0x{pc:08x}.");
                    return Outcome::Breakpoint;
                }
                Some(Step::Unsupported) => {
                    if config.debug {
                        println!("Reached an unimp instruction.");
                    }
                    return Outcome::UnsupportedInstruction;
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::instruction::enc;
    use std::env;

    /// A CPU set up to execute one instruction at a time, so that a test can
    /// state an instruction and look at the machine state it leaves behind.
    struct Harness {
        cpu: Cpu,
        disasm: String,
    }

    impl Harness {
        /// A CPU with a kilobyte of memory, every register zeroed and the
        /// program counter at zero.
        fn new() -> Self {
            Harness {
                cpu: Cpu::new(1),
                disasm: String::new(),
            }
        }

        /// Sets a register, so a test can build up the operands it needs.
        fn set(&mut self, reg: usize, value: u32) -> &mut Self {
            self.cpu.registers[reg] = value;
            self
        }

        /// Puts `bytes` into memory at `address`, for the load and store tests.
        fn poke(&mut self, address: usize, bytes: &[u8]) -> &mut Self {
            self.cpu.memory[address..address + bytes.len()].copy_from_slice(bytes);
            self
        }

        /// Executes one instruction word at address zero.
        fn step(&mut self, word: u32) -> Step {
            self.poke(0, &word.to_le_bytes());
            self.cpu.pc = 0;
            let instruction = Instruction::decode(word);
            self.cpu
                .execute(&instruction, &mut Trace::On(&mut self.disasm))
        }

        /// Executes an instruction and checks the resulting program counter.
        fn step_to(&mut self, word: u32, expected: Step, pc: u32) -> &mut Self {
            assert_eq!(self.step(word), expected);
            assert_eq!(self.cpu.pc, pc, "program counter after {word:#010x}");
            self
        }

        /// The value of a register, after checking that `x0` is still zero.
        fn reg(&self, reg: usize) -> u32 {
            assert_eq!(self.cpu.registers[0], 0, "x0 was written to");
            self.cpu.registers[reg]
        }

        /// The four little-endian bytes at `address`.
        fn word(&self, address: usize) -> u32 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&self.cpu.memory[address..address + 4]);
            u32::from_le_bytes(bytes)
        }

        /// The disassembly of the instruction that was executed last.
        fn disasm(&self) -> &str {
            &self.disasm
        }
    }

    #[test]
    fn register_arithmetic_wraps_instead_of_overflowing() {
        // add x1, x2, x3 and sub x4, x2, x3 at the extremes of the range.
        let mut h = Harness::new();
        h.set(2, u32::MAX).set(3, 1);
        h.step_to(enc::r(0, 3, 2, 0b000, 1), Step::Next, 4);
        assert_eq!(h.reg(1), 0, "u32::MAX + 1");
        h.step_to(enc::r(0b010_0000, 3, 2, 0b000, 4), Step::Next, 4);
        assert_eq!(h.reg(4), u32::MAX - 1, "u32::MAX - 1");
    }

    #[test]
    fn shifts_and_logic() {
        let mut h = Harness::new();
        // x3 holds 6, so the shifts below all move by six places.
        h.set(2, 0b1011).set(3, 0b0110);
        h.step(enc::r(0, 3, 2, 0b001, 1));
        assert_eq!(h.reg(1), 0b1011 << 6, "sll by six places");

        h.set(2, 0x8000_0000);
        h.step(enc::r(0b010_0000, 3, 2, 0b101, 1));
        assert_eq!(h.reg(1), 0xfe00_0000, "sra keeps the sign bit");
        h.step(enc::r(0, 3, 2, 0b101, 1));
        assert_eq!(h.reg(1), 0x0200_0000, "srl moves in zeroes");

        h.set(2, 0b1011).set(3, 0b0110);
        for (funct3, expected) in [(0b100, 0b1101), (0b110, 0b1111), (0b111, 0b0010)] {
            h.step(enc::r(0, 3, 2, funct3, 1));
            assert_eq!(h.reg(1), expected, "funct3 {funct3:#05b}");
        }
    }

    #[test]
    fn register_shifts_use_only_the_low_five_bits_of_the_amount() {
        // The spec defines the amount as rs2[4:0], so 33 has to behave as one
        // and 32 as zero.
        let mut h = Harness::new();
        h.set(2, 0x8000_0000);
        for (amount, sra, srl) in [
            (1u32, 0xc000_0000, 0x4000_0000),
            (32, 0x8000_0000, 0x8000_0000),
            (33, 0xc000_0000, 0x4000_0000),
            (63, 0xffff_ffff, 0x0000_0001),
        ] {
            h.set(3, amount);
            h.step(enc::r(0b010_0000, 3, 2, 0b101, 1));
            assert_eq!(h.reg(1), sra, "sra by {amount}");
            h.step(enc::r(0, 3, 2, 0b101, 1));
            assert_eq!(h.reg(1), srl, "srl by {amount}");
        }
        h.set(2, 1).set(3, 33);
        h.step(enc::r(0, 3, 2, 0b001, 1));
        assert_eq!(h.reg(1), 1 << 1, "sll by 33 shifts by one");
    }

    #[test]
    fn set_less_than_compares_signed_and_unsigned() {
        let mut h = Harness::new();
        // 0x8000_0000 is negative signed, but the largest unsigned value bar one.
        h.set(2, 0x8000_0000).set(3, 1);
        h.step(enc::r(0, 3, 2, 0b010, 1));
        assert_eq!(h.reg(1), 1, "slt: signed, so the left side is smaller");
        h.step(enc::r(0, 3, 2, 0b011, 1));
        assert_eq!(h.reg(1), 0, "sltu: unsigned, so the left side is larger");
    }

    #[test]
    fn immediates_are_applied_as_signed_values() {
        let mut h = Harness::new();
        h.set(2, 1);
        h.step(enc::op_imm(1, 0b000, 2, -1));
        assert_eq!(h.reg(1), 0, "addi 1, -1");
        // andi with a sign extended immediate reaches into the high bits.
        h.set(2, 0xffff_ffff);
        h.step(enc::op_imm(1, 0b111, 2, -2));
        assert_eq!(h.reg(1), 0xffff_fffe, "andi -1, -2");
        h.step(enc::op_imm(1, 0b100, 2, -1));
        assert_eq!(h.reg(1), 0, "xori -1, -1");
    }

    #[test]
    fn upper_immediates() {
        let mut h = Harness::new();
        h.step(enc::lui(1, 0xabcde));
        assert_eq!(h.reg(1), 0xabcd_e000);
        // auipc adds the shifted immediate to this instruction's own pc, which
        // is zero here, not to the pc it is about to become.
        h.step_to(enc::auipc(1, 1), Step::Next, 4);
        assert_eq!(h.reg(1), 0x1000);
    }

    #[test]
    fn loads_sign_extend_the_right_number_of_bytes() {
        // 0x89ab_cdef at address 0x10, loaded by a base register of 0x10.
        let mut h = Harness::new();
        h.set(2, 0x10).poke(0x10, &[0xef, 0xcd, 0xab, 0x89]);
        h.step(enc::load(1, 0b000, 2, 0));
        assert_eq!(h.reg(1), 0xffff_ffef, "lb");
        h.step(enc::load(1, 0b100, 2, 0));
        assert_eq!(h.reg(1), 0x0000_00ef, "lbu");
        h.step(enc::load(1, 0b001, 2, 0));
        assert_eq!(h.reg(1), 0xffff_cdef, "lh");
        h.step(enc::load(1, 0b101, 2, 0));
        assert_eq!(h.reg(1), 0x0000_cdef, "lhu");
        h.step(enc::load(1, 0b010, 2, 0));
        assert_eq!(h.reg(1), 0x89ab_cdef, "lw");
    }

    #[test]
    fn loads_apply_their_offset() {
        let mut h = Harness::new();
        h.set(2, 0x20).poke(0x1e, &[0x11, 0x22, 0x33, 0x44]);
        h.step(enc::load(1, 0b010, 2, -2));
        assert_eq!(h.reg(1), 0x4433_2211, "a negative offset moves down");
    }

    #[test]
    fn stores_write_only_their_own_bytes() {
        let mut h = Harness::new();
        h.set(2, 0x10).set(3, 0x89ab_cdef);
        h.poke(0x10, &[0xff; 4]);
        h.step(enc::store(0b000, 2, 3, 0));
        assert_eq!(h.word(0x10), 0xffff_ffef, "sb leaves the rest alone");
        h.step(enc::store(0b001, 2, 3, 0));
        assert_eq!(h.word(0x10), 0xffff_cdef, "sh writes two bytes");
        h.step(enc::store(0b010, 2, 3, 0));
        assert_eq!(h.word(0x10), 0x89ab_cdef, "sw writes all four");
    }

    #[test]
    fn branches_move_the_pc_only_when_taken() {
        let mut h = Harness::new();
        h.set(2, 7).set(3, 7);
        h.step_to(enc::branch(0b000, 2, 3, 8), Step::Jump, 8);
        h.step_to(enc::branch(0b001, 2, 3, 8), Step::Next, 4);
        // bltu and bgeu compare without sign extension.
        h.set(2, -1i32 as u32);
        h.set(3, 1);
        h.step_to(enc::branch(0b110, 2, 3, 4), Step::Next, 4);
        h.step_to(enc::branch(0b100, 2, 3, 4), Step::Jump, 4);
    }

    #[test]
    fn jal_links_and_jumps() {
        let mut h = Harness::new();
        h.step_to(enc::jal(1, 0x1000), Step::Jump, 0x1000);
        assert_eq!(h.reg(1), 4, "the link register holds the fall-through pc");
    }

    #[test]
    fn jalr_links_jumps_and_clears_the_low_bit() {
        let mut h = Harness::new();
        h.set(2, 0x1003);
        h.step_to(enc::jalr(1, 2, 4), Step::Jump, 0x1006);
        assert_eq!(h.reg(1), 4);
        h.set(2, 0x1003);
        h.step_to(enc::jalr(1, 2, 0), Step::Jump, 0x1002);
    }

    #[test]
    fn jalr_reads_its_base_before_writing_the_link_register() {
        // `jalr x1, x1, 0` is a legal way of branching on a register.
        let mut h = Harness::new();
        h.set(1, 0x40);
        h.step_to(enc::jalr(1, 1, 0), Step::Jump, 0x40);
        assert_eq!(h.reg(1), 4);
    }

    #[test]
    fn a_nop_is_a_no_op() {
        let mut h = Harness::new();
        h.step_to(enc::op_imm(0, 0b000, 0, 0), Step::Next, 4);
        assert_eq!(h.disasm(), "nop");
    }

    #[test]
    fn ecall_and_ebreak_are_reported_to_the_run_loop() {
        let mut h = Harness::new();
        h.step_to(enc::system(0, 0b000, 0, 0x000), Step::Ecall, 4);
        assert_eq!(h.disasm(), "ecall");
        // EBREAK is a requested trap, so it has to stop the run like ECALL
        // does rather than retire as if nothing had happened.
        h.step_to(enc::system(0, 0b000, 0, 0x001), Step::Breakpoint, 4);
        assert_eq!(h.disasm(), "ebreak");
    }

    #[test]
    fn mret_is_a_no_op() {
        // It belongs to the privileged architecture, which is not implemented.
        let mut h = Harness::new();
        h.step_to(enc::system(0, 0b000, 0, 0x302), Step::Next, 4);
        assert_eq!(h.disasm(), "mret");
    }

    #[test]
    fn fence_is_a_no_op_and_a_zero_word_is_unsupported() {
        let mut h = Harness::new();
        h.step_to(enc::fence(0b000), Step::Next, 4);
        assert_eq!(h.disasm(), "fence");
        h.step_to(0, Step::Unsupported, 4);
        assert_eq!(h.disasm(), "unimp");
    }

    #[test]
    fn writing_x0_never_changes_it() {
        let mut h = Harness::new();
        h.set(2, 5);
        h.step(enc::r(0, 2, 0, 0b000, 0));
        assert_eq!(h.reg(0), 0);
    }

    #[test]
    fn disassembly_names_the_operands() {
        let mut h = Harness::new();
        h.set(2, 0x10).set(3, 0x20);
        h.step(enc::r(0b010_0000, 3, 2, 0b000, 1));
        assert_eq!(h.disasm(), "sub     x1,x2,x3");
        h.step(enc::op_imm(1, 0b000, 2, -4));
        assert_eq!(h.disasm(), "addi    x1,x2,-4");
        h.step(enc::load(1, 0b010, 2, -4));
        assert_eq!(h.disasm(), "lw      x1,-4(x2)");
        h.step(enc::store(0b010, 2, 3, -4));
        assert_eq!(h.disasm(), "sw      x3,-4(x2)");
        h.step(enc::branch(0b000, 2, 3, -8));
        assert_eq!(h.disasm(), "beq     x2,x3,fffffff8");
        h.step(enc::lui(1, 0xabcde));
        assert_eq!(h.disasm(), "lui     x1,0xabcde");
    }

    #[test]
    fn the_trace_and_the_listing_print_the_same_disassembly() {
        // The reason the disassembly lives in one place rather than in each arm
        // of `execute`: a line of a trace is a line of a listing, so neither can
        // end up naming an instruction differently from the other. The harness
        // executes each word at address zero, which is where the listing would
        // print it, and the base register points into memory so that the load
        // and the store have somewhere to go.
        let mut h = Harness::new();
        h.set(2, 0x10);
        for word in [
            enc::r(0b010_0000, 3, 2, 0b000, 1),
            enc::op_imm(1, 0b000, 2, -4),
            enc::op_imm(0, 0b000, 0, 0),
            enc::load(1, 0b100, 2, -1),
            enc::store(0b010, 2, 3, -4),
            enc::branch(0b000, 2, 3, -8),
            enc::jal(1, 0x1000),
            enc::jalr(1, 2, -4),
            enc::lui(1, 0xabcde),
            enc::auipc(1, 0xabcde),
            enc::system(1, 0b001, 2, 0x300),
            enc::system(0, 0b000, 0, 0x000),
            enc::fence(0b001),
            0,
        ] {
            h.step(word);
            let listed = Disassembly::new(0, &Instruction::decode(word)).to_string();
            assert_eq!(h.disasm(), listed, "{word:#010x}");
        }
    }

    #[test]
    fn commands_report_the_machine_state() {
        let mut h = Harness::new();
        h.set(2, 0x1234_5678).poke(0x14c, &[0x13, 0x01, 0x00, 0x00]);
        let cpu = &h.cpu;
        assert_eq!(cpu.command("reg 2").as_deref(), Some("0x12345678"));
        assert_eq!(cpu.command("reg 0").as_deref(), Some("0x0"));
        assert_eq!(cpu.command("mem 14c").as_deref(), Some("0x00000113"));
        // Extra words after the argument are ignored.
        assert_eq!(cpu.command("reg  2  ").as_deref(), Some("0x12345678"));
    }

    #[test]
    fn a_command_without_an_argument_is_an_error_not_a_crash() {
        let cpu = Harness::new().cpu;
        for (input, expected) in [
            ("reg", "bad argument: expected a register number"),
            ("mem", "bad argument: expected a hex address"),
            ("reg abc", "bad argument: invalid digit found in string"),
            ("reg 99", "bad argument: no such register"),
            ("mem ffffffff", "bad argument: memory out of bounds"),
            ("mem zzz", "bad argument: invalid digit found in string"),
            ("nonsense", "Unknown command: nonsense"),
        ] {
            assert_eq!(cpu.command(input).as_deref(), Some(expected), "{input}");
        }
    }

    #[test]
    fn an_empty_command_has_no_reply() {
        // An empty line is the run loop's cue to step, not a command.
        let cpu = Harness::new().cpu;
        assert_eq!(cpu.command(""), None);
        assert_eq!(cpu.command("   "), None);
    }

    #[test]
    fn loading_reports_why_it_failed() {
        // No such file.
        let err = Cpu::new(16).load("does/not/exist").unwrap_err();
        assert!(matches!(err, LoadError::Io(_)), "{err}");

        // A file that is not an ELF image at all.
        let err = Cpu::new(16)
            .load(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
            .unwrap_err();
        assert!(matches!(err, LoadError::Malformed), "{err}");

        // A well formed ELF image, but for another architecture: the test
        // binary itself is a native executable.
        let err = Cpu::new(1024)
            .load(env::current_exe().unwrap())
            .unwrap_err();
        assert!(matches!(err, LoadError::WrongArchitecture), "{err}");

        // A correct image that does not fit in the configured memory.
        let err = Cpu::new(1)
            .load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/add"))
            .unwrap_err();
        assert!(
            matches!(err, LoadError::ImageTooLarge { size, capacity }
                if size > capacity),
            "{err}"
        );
    }

    #[test]
    fn loading_points_the_program_counter_at_the_entry_point() {
        let mut cpu = Cpu::new(16);
        cpu.load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/add"))
            .unwrap();
        assert_eq!(cpu.pc, 0x1000);
    }

    #[test]
    fn a_listing_covers_the_whole_of_the_code_segment() {
        // The segment holding the entry point of tests/add: 0x6bc bytes of code
        // at file offset 0x1000, which is 431 instructions.
        let mut cpu = Cpu::new(16);
        cpu.load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/add"))
            .unwrap();
        let listing: Vec<String> = cpu.disassembly().collect();
        assert_eq!(listing.len(), 0x6bc / 4);
        // The first line is the entry point, which jumps over the machine-mode
        // setup that a trace of a run never shows.
        assert_eq!(
            listing[0],
            "00001000:   0480006f          \tjal     x0,00001048",
        );
    }

    #[test]
    fn a_cpu_with_no_image_has_nothing_to_list() {
        let mut cpu = Cpu::new(16);
        assert_eq!(cpu.disassembly().count(), 0);
        // The listing is empty until an image says where the code is, and loading
        // one is what fills it in.
        cpu.load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/simple"))
            .unwrap();
        assert!(cpu.disassembly().next().is_some());
    }

    #[test]
    fn a_breakpoint_ends_the_run() {
        // `run` needs no image to have been loaded to execute one instruction.
        let mut cpu = Cpu::new(1);
        cpu.memory[..4].copy_from_slice(&enc::system(0, 0b000, 0, 0x001).to_le_bytes());
        assert_eq!(cpu.run(&RunConfig::default()), Outcome::Breakpoint);
    }

    /// A CPU with `text` in memory and a system call set up to run.
    fn syscall(number: u32, a0: u32, a1: u32, a2: u32, text: &[u8]) -> Cpu {
        let mut cpu = Cpu::new(1);
        let start = 0x100;
        cpu.memory[start..start + text.len()].copy_from_slice(text);
        cpu.registers[A0] = a0;
        cpu.registers[A1] = a1;
        cpu.registers[A2] = a2;
        cpu.registers[A7] = number;
        cpu
    }

    #[test]
    fn a_guest_read_is_bounded_by_memory() {
        let cpu = Cpu::new(1);
        let end = cpu.memory.len() as u32;
        assert_eq!(cpu.guest_bytes(0x10, 4).map(<[u8]>::len), Some(4));
        // An empty read at the very end is empty, not out of bounds.
        assert_eq!(cpu.guest_bytes(end, 0).map(<[u8]>::len), Some(0));
        // Reaching exactly the end is fine; one byte further is not.
        assert!(cpu.guest_bytes(end - 4, 4).is_some());
        assert_eq!(cpu.guest_bytes(end - 4, 5), None);
        assert_eq!(cpu.guest_bytes(end + 1, 0), None);
        // A length that would overflow a usize cannot wrap into a valid range.
        assert_eq!(cpu.guest_bytes(0x10, u32::MAX), None);
    }

    #[test]
    fn write_returns_the_number_of_bytes_it_wrote() {
        let mut cpu = syscall(WRITE_SYSCALL, 1, 0x100, 6, b"hello\n");
        assert_eq!(cpu.handle_ecall(false), None, "write does not end the run");
        assert_eq!(cpu.registers[A0], 6);
    }

    #[test]
    fn write_reports_a_buffer_outside_memory() {
        let end = (Cpu::new(1).memory.len() + 0x100) as u32;
        let mut cpu = syscall(WRITE_SYSCALL, 1, 0x100, 4, b"four");
        // Straddling the end of memory is as much a fault as starting past it.
        cpu.registers[A2] = end - 0x100 + 1;
        assert_eq!(cpu.handle_ecall(false), None);
        assert_eq!(cpu.registers[A0], SYSCALL_ERROR);

        let mut cpu = syscall(WRITE_SYSCALL, 1, u32::MAX, 1, b"x");
        assert_eq!(cpu.handle_ecall(false), None);
        assert_eq!(cpu.registers[A0], SYSCALL_ERROR);
    }

    #[test]
    fn exit_ends_the_run_and_an_unknown_call_does_too() {
        let mut cpu = syscall(EXIT_SYSCALL, 7, 0, 0, b"");
        assert_eq!(cpu.handle_ecall(false), Some(Outcome::Exited(7)));
        let mut cpu = syscall(42, 0, 0, 0, b"");
        assert_eq!(
            cpu.handle_ecall(false),
            Some(Outcome::UnsupportedSyscall(42))
        );
    }
}

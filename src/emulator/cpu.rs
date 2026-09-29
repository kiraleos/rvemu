use super::instruction::{sign_extend, Instruction};
use crate::Args;
use elf_rs::{Elf, ElfFile};
use std::fmt::Write as _;
use std::io::{Read, Write};

/// The ABI name of every register, indexed by register number.
const ALIASES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

/// Number of general purpose registers, `x0` through `x31`.
const REGISTER_COUNT: usize = 32;

/// `x2`, the stack pointer.
const SP: usize = 2;
/// `x10`, the first argument register, and where `exit` reads its status.
const A0: usize = 10;
/// `x17`, the register holding the system call number.
const A7: usize = 17;

/// Every RV32I instruction is four bytes wide.
const INSTRUCTION_SIZE: usize = 4;

/// The spec defines a shift amount as the low five bits of its operand, whether
/// that comes from a register (`sll`) or from an immediate (`slli`).
const SHIFT_AMOUNT_MASK: u32 = 0b1_1111;

/// A RISC-V RV32I CPU, as described by the unprivileged base integer spec.
pub struct Cpu {
    memory: Vec<u8>,
    registers: [u32; REGISTER_COUNT],
    pc: u32,
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
    /// An instruction this emulator does not implement, so it should stop.
    Unsupported,
}

impl Cpu {
    /// Creates a CPU with `mem_size` kilobytes of memory and every register
    /// zeroed.
    pub fn new(mem_size: usize) -> Self {
        Cpu {
            memory: vec![0; mem_size * 1024],
            registers: [0; REGISTER_COUNT],
            pc: 0,
        }
    }

    /// Loads an ELF image into memory and points the program counter at its
    /// entry point.
    pub fn load(&mut self, path: &str) {
        let mut elf_file = std::fs::File::open(path).expect("open file failed");
        let mut elf_buf = Vec::<u8>::new();
        elf_file
            .read_to_end(&mut elf_buf)
            .expect("read file failed");
        let elf = Elf::from_bytes(&elf_buf).expect("Are you sure this is an ELF file?");
        match elf.elf_header().machine() {
            elf_rs::ElfMachine::RISC_V => {
                for phdr in elf.program_header_iter() {
                    let e_entry = elf.entry_point();
                    if phdr.vaddr() <= e_entry && e_entry < phdr.vaddr() + phdr.memsz() {
                        let p_vaddr = phdr.vaddr();
                        let p_offset = phdr.offset();
                        self.pc = (e_entry - p_vaddr + p_offset)
                            .try_into()
                            .expect("couldn't convert u64 entry addr to u32");
                    }
                }
            }
            _ => {
                panic!(
                    "unsupported architecture: {:#?}",
                    elf.elf_header().machine()
                );
            }
        }
        self.memory[..elf_buf.len()].copy_from_slice(&elf_buf);
    }

    /// Prints the program counter and every register, four to a line.
    ///
    /// `aliases` selects between the ABI names (`sp`, `a0`, ...) and the
    /// numeric ones (`x2`, `x10`, ...).
    pub fn print_registers(&self, aliases: bool) {
        let mut reg_name;
        println!(" pc: 0x{:0>8x}", self.pc);
        let mut strbuilder = String::new();
        for (i, alias) in ALIASES.iter().enumerate() {
            if aliases {
                strbuilder += &*format!("{:>4}: 0x{:0>8x}  ", *alias, self.registers[i]);
            } else {
                reg_name = String::from("x") + &i.to_string();
                strbuilder += &*format!("{:>3}: 0x{:0>8x}    ", reg_name, self.registers[i]);
            }
            if (i + 1) % 4 == 0 {
                strbuilder += "\n";
            }
        }
        println!("{}", strbuilder);
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
        self.load_bytes::<INSTRUCTION_SIZE>(self.pc)
    }

    /// Executes `inst`, updating the registers and the program counter.
    ///
    /// The disassembly that `--debug` prints is written to `disasm`, which the
    /// caller owns so that the run loop can reuse one allocation instead of
    /// formatting into a fresh `String` for every instruction.
    fn execute(&mut self, inst: &Instruction, disasm: &mut String) -> Step {
        disasm.clear();

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
                let (mnemonic, value) = match funct3 {
                    // `add` and `sub` share funct3 and are told apart by funct7.
                    0b000 => match funct7 {
                        0b000_0000 => ("add", a.wrapping_add(b)),
                        0b010_0000 => ("sub", a.wrapping_sub(b)),
                        other => panic!("unknown R funct7: {other:#09b}"),
                    },
                    0b001 => ("sll", a << (b & SHIFT_AMOUNT_MASK)),
                    0b010 => ("slt", ((a as i32) < (b as i32)) as u32),
                    0b011 => ("sltu", (a < b) as u32),
                    0b100 => ("xor", a ^ b),
                    // `srl` and `sra`, like `add` and `sub`, share funct3.
                    0b101 => match funct7 {
                        0b000_0000 => ("srl", a >> (b & SHIFT_AMOUNT_MASK)),
                        0b010_0000 => ("sra", ((a as i32) >> (b & SHIFT_AMOUNT_MASK)) as u32),
                        other => panic!("unknown R funct7: {other:#09b}"),
                    },
                    0b110 => ("or", a | b),
                    0b111 => ("and", a & b),
                    other => {
                        panic!("execute: unimplemented R funct3: {other:#05b}")
                    }
                };
                let _ = write!(disasm, "{mnemonic:<8}x{rd},x{rs1},x{rs2}");
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
                    // The shifts are the only OP-IMM instructions that print
                    // their operand in hex, and the only ones that have to
                    // check the funct7 half of the immediate.
                    0b001 | 0b101 => {
                        let shamt = imm & SHIFT_AMOUNT_MASK;
                        let (mnemonic, value) = if funct3 == 0b001 {
                            ("slli", a << shamt)
                        } else {
                            match imm >> 5 & 0b111_1111 {
                                0b000_0000 => ("srli", a >> shamt),
                                0b010_0000 => {
                                    // Shifting right and then refilling the
                                    // vacated high bits with copies of the
                                    // sign bit is an arithmetic shift.
                                    ("srai", sign_extend(a >> shamt, 32 - shamt))
                                }
                                other => panic!("unknown shift funct7: {other:#09b}"),
                            }
                        };
                        let _ = write!(disasm, "{mnemonic:<8}x{rd},x{rs1},{shamt:#x}");
                        self.registers[rd] = value;
                    }
                    _ => {
                        let (mnemonic, value) = match funct3 {
                            0b000 => ("addi", a.wrapping_add(imm)),
                            0b010 => ("slti", ((a as i32) < imm as i32) as u32),
                            0b011 => ("sltiu", (a < imm) as u32),
                            0b100 => ("xori", a ^ imm),
                            0b110 => ("ori", a | imm),
                            0b111 => ("andi", a & imm),
                            other => panic!("unknown I funct3: {other:#05b}"),
                        };
                        // `sltiu` compares its register against the
                        // sign-extended immediate read as an unsigned value,
                        // and is displayed that way; the other OP-IMM forms
                        // display a signed immediate.
                        let _ = if funct3 == 0b011 {
                            write!(disasm, "{mnemonic:<8}x{rd},x{rs1},{imm}")
                        } else {
                            write!(disasm, "{mnemonic:<8}x{rd},x{rs1},{}", imm as i32)
                        };
                        // `addi x0, x0, 0` is the canonical encoding of a nop.
                        if rd == 0 && rs1 == 0 && imm == 0 {
                            disasm.clear();
                            disasm.push_str("nop");
                        }
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
                match funct3 {
                    0b000 => {
                        let _ = write!(disasm, "lb      x{rd},{}(x{rs1})", imm as i32);
                        self.registers[rd] = sign_extend(self.load_bytes::<1>(address), 8);
                    }
                    0b001 => {
                        let _ = write!(disasm, "lh      x{rd},{}(x{rs1})", imm as i32);
                        self.registers[rd] = sign_extend(self.load_bytes::<2>(address), 16);
                    }
                    0b010 => {
                        let _ = write!(disasm, "lw      x{rd},{}(x{rs1})", imm as i32);
                        self.registers[rd] = self.load_bytes::<4>(address);
                    }
                    0b100 => {
                        let _ = write!(disasm, "lbu     x{rd},{imm}(x{rs1})");
                        self.registers[rd] = self.load_bytes::<1>(address);
                    }
                    0b101 => {
                        let _ = write!(disasm, "lhu     x{rd},{imm}(x{rs1})");
                        self.registers[rd] = self.load_bytes::<2>(address);
                    }
                    other => panic!("unknown I funct3: {other:#05b}"),
                }
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
                let mnemonic = match funct3 {
                    0b000 => {
                        self.store_bytes::<1>(address, value);
                        "sb"
                    }
                    0b001 => {
                        self.store_bytes::<2>(address, value);
                        "sh"
                    }
                    0b010 => {
                        self.store_bytes::<4>(address, value);
                        "sw"
                    }
                    other => panic!("unknown S funct3: {other:#05b}"),
                };
                let _ = write!(disasm, "{mnemonic:<8}x{rs2},{}(x{rs1})", imm as i32);
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
                let (mnemonic, taken) = match funct3 {
                    0b000 => ("beq", a == b),
                    0b001 => ("bne", a != b),
                    0b100 => ("blt", (a as i32) < (b as i32)),
                    0b101 => ("bge", (a as i32) >= (b as i32)),
                    0b110 => ("bltu", a < b),
                    0b111 => ("bgeu", a >= b),
                    other => {
                        panic!("execute: unimplemented B funct3: {other:#05b}")
                    }
                };
                let _ = write!(disasm, "{mnemonic:<8}x{rs1},x{rs2},{target:08x}");
                if taken {
                    self.pc = target as u32;
                    Step::Jump
                } else {
                    Step::Next
                }
            }
            Instruction::Jump { rd, imm } => {
                let _ = write!(disasm, "jal     x{rd},{imm:08x}");
                self.registers[rd] = self.pc.wrapping_add(4);
                self.pc = self.pc.wrapping_add(imm);
                Step::Jump
            }
            Instruction::JumpRegister { rd, rs1, imm } => {
                let _ = write!(disasm, "jalr    x{rd},x{rs1},{imm:#x}");
                // Read the base before writing rd: `jalr x1, x1, 0` is legal.
                // The spec requires the low bit of the target to be zero.
                let target = self.registers[rs1].wrapping_add(imm) & !1;
                self.registers[rd] = self.pc.wrapping_add(4);
                self.pc = target;
                Step::Jump
            }
            Instruction::Lui { rd, imm } => {
                let _ = write!(disasm, "lui     x{rd},{imm:#x}");
                self.registers[rd] = imm << 12;
                Step::Next
            }
            Instruction::Auipc { rd, imm } => {
                let _ = write!(disasm, "auipc   x{rd},{imm:#x}");
                self.registers[rd] = self.pc.wrapping_add(imm << 12);
                Step::Next
            }
            Instruction::System {
                rd,
                rs1,
                imm,
                funct3,
            } => match funct3 {
                // `ecall`, `ebreak` and `mret` share funct3 and are told apart
                // by the immediate, which for them is a plain 12-bit field
                // rather than a sign extended one.
                0b000 => match imm {
                    0x000 => {
                        disasm.push_str("ecall");
                        Step::Ecall
                    }
                    0x001 => {
                        disasm.push_str("ebreak");
                        Step::Next
                    }
                    0b0011_0000_0010 => {
                        disasm.push_str("mret");
                        Step::Next
                    }
                    other => panic!("unknown I imm: {other:#014b}"),
                },
                // This emulator implements no CSR file, so the CSR instructions
                // decode and retire without touching any register.
                0b001 => {
                    let _ = write!(disasm, "csrrw   x{rd},{imm:#x},x{rs1}");
                    Step::Next
                }
                0b010 => {
                    let _ = write!(disasm, "csrrs   x{rd},{imm:#x},x{rs1}");
                    Step::Next
                }
                0b011 => {
                    let _ = write!(disasm, "csrrc   x{rd},{imm:#x},x{rs1}");
                    Step::Next
                }
                0b101 => {
                    let _ = write!(disasm, "csrrwi  x{rd},{imm:#x},{rs1}");
                    Step::Next
                }
                0b110 => {
                    let _ = write!(disasm, "csrrsi  x{rd},{imm:#x},{rs1}");
                    Step::Next
                }
                0b111 => {
                    let _ = write!(disasm, "csrrci  x{rd},{imm:#x},{rs1}");
                    Step::Next
                }
                other => panic!("unknown I funct3: {other:#05b}"),
            },
            Instruction::Fence => {
                disasm.push_str("fence");
                Step::Next
            }
            Instruction::Unsupported => {
                disasm.push_str("unimp");
                Step::Unsupported
            }
        };

        self.registers[0] = 0;
        if step != Step::Jump {
            self.pc = self.pc.wrapping_add(INSTRUCTION_SIZE as u32);
        }
        step
    }

    fn command_handler(&mut self, com: &str) {
        if com.is_empty() {
            return;
        }
        let tokens: Vec<&str> = com.split(' ').collect();
        match tokens[0] {
            "mem" => {
                let addr = usize::from_str_radix(tokens[1], 16);
                match addr {
                    Ok(addr) => {
                        if addr + 3 > self.memory.len() - 1 {
                            println!("bad argument: memory out of bounds");
                            return;
                        }
                        let chunk = self.load_bytes::<4>(addr as u32);
                        println!("{:#010x}", chunk)
                    }
                    Err(err) => println!("bad argument: {}", err),
                }
            }
            "reg" => {
                let reg = tokens[1].parse::<usize>();
                match reg {
                    Ok(reg) => {
                        if reg > self.registers.len() - 1 {
                            println!("bad argument: no such register");
                            return;
                        }
                        println!("{:#x}", self.registers[reg])
                    }
                    Err(err) => println!("bad argument: {}", err),
                }
            }
            _ => {
                println!("Unknown command: {}", tokens[0])
            }
        }
    }

    fn run_interactive(&mut self, args: Args) -> i32 {
        let ret: i32;
        let pc = args.pc;
        if let Some(pc) = pc {
            self.pc = u32::from_str_radix(&pc, 16).unwrap_or(self.pc);
        }
        if args.stack {
            self.registers[SP] = (self.memory.len() - 1) as u32;
        }
        let mut buf = String::new();
        let mut disasm = String::new();
        loop {
            buf.clear();
            print!("> ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut buf).unwrap();
            buf.pop();
            self.command_handler(&buf);

            let raw_inst = self.fetch();
            let inst = Instruction::decode(raw_inst);
            let pc_copy = self.pc;

            // An empty line steps the program; anything else was a command
            // that has already been handled.
            let mut step = None;
            if buf.is_empty() {
                step = Some(self.execute(&inst, &mut disasm));
                if args.registers {
                    self.print_registers(args.aliases);
                }
                println!("{:<08x}:   {:08x}          	{}", pc_copy, raw_inst, disasm);
            }

            if (self.pc as usize) >= self.memory.len() {
                if args.debug {
                    println!("PC overflow.");
                }
                ret = -1;
                break;
            }
            match step {
                Some(Step::Ecall) => match self.registers[A7] {
                    // `exit` syscall
                    93 => {
                        ret = self.registers[A0] as i32;
                        println!("Program exited with exit code: {}", ret);
                        break;
                    }
                    _ => {
                        if args.debug {
                            println!("Unimplemented ECALL: {}", self.registers[A7],);
                        }
                        ret = -2;
                        break;
                    }
                },
                Some(Step::Unsupported) => {
                    if args.debug {
                        println!("Reached an unimp instruction.");
                    }
                    ret = -3;
                    break;
                }
                _ => {}
            }
        }
        ret
    }

    pub fn run(&mut self, args: Args) -> i32 {
        if args.interactive {
            return self.run_interactive(args);
        }
        let ret: i32;
        let pc = args.pc;
        if let Some(pc) = pc {
            self.pc = u32::from_str_radix(&pc, 16).unwrap_or(self.pc);
        }
        if args.stack {
            self.registers[SP] = (self.memory.len() - 1) as u32;
        }
        let mut disasm = String::new();
        loop {
            if args.registers {
                self.print_registers(args.aliases);
            }
            let raw_inst = self.fetch();
            let inst = Instruction::decode(raw_inst);
            let pc_copy = self.pc;
            let step = self.execute(&inst, &mut disasm);
            if args.debug {
                println!("{:<08x}:   {:08x}          	{}", pc_copy, raw_inst, disasm);
            }

            if (self.pc as usize) >= self.memory.len() {
                if args.debug {
                    println!("PC overflow.");
                }
                ret = -1;
                break;
            }
            match step {
                Step::Ecall => match self.registers[A7] {
                    // `exit` syscall
                    93 => {
                        ret = self.registers[A0] as i32;
                        println!("Program exited with exit code: {}", ret);
                        break;
                    }
                    _ => {
                        if args.debug {
                            println!("Unimplemented ECALL: {}", self.registers[A7],);
                        }
                        ret = -2;
                        break;
                    }
                },
                Step::Unsupported => {
                    if args.debug {
                        println!("Reached an unimp instruction.");
                    }
                    ret = -3;
                    break;
                }
                _ => {}
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::instruction::enc;

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
            self.cpu.execute(&instruction, &mut self.disasm)
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
    fn ecall_ebreak_and_mret_report_what_the_run_loop_has_to_do() {
        let mut h = Harness::new();
        h.step_to(enc::system(0, 0b000, 0, 0x000), Step::Ecall, 4);
        assert_eq!(h.disasm(), "ecall");
        h.step_to(enc::system(0, 0b000, 0, 0x001), Step::Next, 4);
        assert_eq!(h.disasm(), "ebreak");
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
}

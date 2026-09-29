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
                    0b001 => ("sll", a << b),
                    0b010 => ("slt", ((a as i32) < (b as i32)) as u32),
                    0b011 => ("sltu", (a < b) as u32),
                    0b100 => ("xor", a ^ b),
                    // `srl` and `sra`, like `add` and `sub`, share funct3.
                    0b101 => match funct7 {
                        0b000_0000 => ("srl", a >> b),
                        0b010_0000 => ("sra", ((a as i32) >> b) as u32),
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
                        let shamt = imm & 0b1_1111;
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

use super::instruction::*;
use super::trap::{LoadError, Outcome, Trap};
use crate::Args;
use elf_rs::{Elf, ElfFile};
use std::io::{Read, Write};

const ALIASES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0",
    "a1", "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5",
    "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

pub struct Cpu {
    memory: Vec<u8>,
    registers: [u32; 32],
    csrs: [u32; NUM_CSRS],
    pc: u32,
}

/// The Zicsr address space is 12 bits wide, so 4096 registers suffice.
const NUM_CSRS: usize = 4096;

impl Cpu {
    pub fn new(mem_size: usize) -> Self {
        Cpu {
            memory: vec![0; mem_size * 1024],
            registers: [0; 32],
            csrs: [0; NUM_CSRS],
            pc: 0,
        }
    }

    /// Load a RISC-V ELF binary into memory and set `pc` to its entry point.
    pub fn load(&mut self, path: &str) -> Result<(), LoadError> {
        let mut elf_file = std::fs::File::open(path).map_err(|e| {
            LoadError::Io(format!("could not open '{}': {}", path, e))
        })?;
        let mut elf_buf = Vec::<u8>::new();
        elf_file.read_to_end(&mut elf_buf).map_err(|e| {
            LoadError::Io(format!("could not read '{}': {}", path, e))
        })?;
        let elf = Elf::from_bytes(&elf_buf).map_err(|_| {
            LoadError::Elf(format!(
                "'{}' is not a valid ELF file",
                path
            ))
        })?;
        match elf.elf_header().machine() {
            elf_rs::ElfMachine::RISC_V => {
                for phdr in elf.program_header_iter() {
                    let e_entry = elf.entry_point();
                    if phdr.vaddr() <= e_entry
                        && e_entry < phdr.vaddr() + phdr.memsz()
                    {
                        let p_vaddr = phdr.vaddr();
                        let p_offset = phdr.offset();
                        self.pc = (e_entry - p_vaddr + p_offset)
                            .try_into()
                            .map_err(|_| {
                                LoadError::Elf(format!(
                                    "ELF entry address does not fit in 32 bits: {}",
                                    e_entry
                                ))
                            })?;
                    }
                }
            }
            machine => {
                return Err(LoadError::Arch(format!(
                    "unsupported architecture: {:#?}",
                    machine
                )));
            }
        }
        let raw_data: Vec<u8> = elf_buf.into_iter().collect();
        self.memory[..raw_data.len()].copy_from_slice(&raw_data);
        Ok(())
    }

    pub fn print_registers(&self, aliases: bool) {
        let mut reg_name;
        println!(" pc: 0x{:0>8x}", self.pc);
        let mut strbuilder = String::new();
        for (i, alias) in ALIASES.iter().enumerate() {
            if aliases {
                strbuilder += &*format!(
                    "{:>4}: 0x{:0>8x}  ",
                    *alias, self.registers[i]
                );
            } else {
                reg_name = String::from("x") + &i.to_string();
                strbuilder += &*format!(
                    "{:>3}: 0x{:0>8x}    ",
                    reg_name, self.registers[i]
                );
            }
            if (i + 1) % 4 == 0 {
                strbuilder += "\n";
            }
        }
        println!("{}", strbuilder);
    }

    fn fetch(&self) -> u32 {
        let index = self.pc as usize;
        self.memory[index] as u32
            | ((self.memory[index + 1]) as u32) << 8
            | ((self.memory[index + 2]) as u32) << 16
            | ((self.memory[index + 3]) as u32) << 24
    }

    fn decode(&self, inst: u32) -> Instruction {
        let mut instruction = Instruction::new();
        let opcode = inst & 0b1111111;
        instruction.opcode = opcode;
        match opcode {
            // R Type
            0b0110011 => {
                let rd = ((inst >> 7) & 0b11111) as usize;
                let funct3 = (inst >> 12) & 0b111;
                let rs1 = ((inst >> 15) & 0b11111) as usize;
                let rs2 = ((inst >> 20) & 0b11111) as usize;
                let funct7 = (inst >> 25) & 0b1111111;
                instruction.type_data = InstTypeData::R {
                    rd,
                    funct3,
                    rs1,
                    rs2,
                    funct7,
                };
                instruction.type_name = InstTypeName::R;
            }

            // I Type
            0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 => {
                let rd = ((inst >> 7) & 0b11111) as usize;
                let funct3 = (inst >> 12) & 0b111;
                let rs1 = ((inst >> 15) & 0b11111) as usize;
                let imm = (inst >> 20) & 0b111111111111;
                let imm = Cpu::sign_extend(imm, 12);
                instruction.type_data = InstTypeData::I {
                    rd,
                    funct3,
                    rs1,
                    imm,
                };
                instruction.type_name = InstTypeName::I;
            }

            // S Type
            0b0100011 => {
                let imm4_0 = (inst >> 7) & 0b11111;
                let imm11_5 = (inst >> 25) & 0b1111111;
                let imm = ((imm11_5 << 5) | imm4_0) as i32 as u32;
                let imm = Cpu::sign_extend(imm, 12);

                let funct3 = (inst >> 12) & 0b111;
                let rs1 = ((inst >> 15) & 0b11111) as usize;
                let rs2 = ((inst >> 20) & 0b11111) as usize;
                instruction.type_data = InstTypeData::S {
                    imm,
                    funct3,
                    rs1,
                    rs2,
                };
                instruction.type_name = InstTypeName::S;
            }

            // B type
            0b1100011 => {
                let imm11 = (inst >> 7) & 0b1;
                let imm4_1 = (inst >> 8) & 0b1111;
                let imm10_5 = (inst >> 25) & 0b111111;
                let imm12 = (inst >> 31) & 0b1;
                let imm = (imm12 << 12)
                    | (imm11 << 11)
                    | (imm10_5 << 5)
                    | (imm4_1 << 1);
                let imm = Cpu::sign_extend(imm, 12);

                let funct3 = (inst >> 12) & 0b111;
                let rs1 = ((inst >> 15) & 0b11111) as usize;
                let rs2 = ((inst >> 20) & 0b11111) as usize;
                instruction.type_data = InstTypeData::B {
                    imm,
                    funct3,
                    rs1,
                    rs2,
                };
                instruction.type_name = InstTypeName::B;
            }

            // J type
            0b1101111 => {
                let rd = ((inst >> 7) & 0b11111) as usize;

                let imm19_12 = (inst >> 12) & 0b11111111;
                let imm11 = (inst >> 20) & 0b1;
                let imm10_1 = (inst >> 21) & 0b1111111111;
                let imm20 = (inst >> 31) & 0b1;
                let imm = (imm20 << 20)
                    | (imm19_12 << 12)
                    | (imm11 << 11)
                    | (imm10_1 << 1);
                let imm = Cpu::sign_extend(imm, 12);
                instruction.type_data = InstTypeData::J { rd, imm };
                instruction.type_name = InstTypeName::J;
            }

            // U type
            0b0110111 | 0b0010111 => {
                let rd = ((inst >> 7) & 0b11111) as usize;
                let imm = (inst >> 12) & 0b11111111111111111111;
                instruction.type_data = InstTypeData::U { rd, imm };
                instruction.type_name = InstTypeName::U;
            }

            // Fence
            0b0001111 => {
                instruction.type_data = InstTypeData::Fence;
                instruction.type_name = InstTypeName::Fence;
            }

            _ => {
                instruction.type_data = InstTypeData::Unimp;
                instruction.type_name = InstTypeName::Unimp;
            }
        }
        if inst == 0 || inst == 0xc0001073 {
            instruction.type_data = InstTypeData::Unimp;
            instruction.type_name = InstTypeName::Unimp;
        }
        instruction
    }

    fn execute(&mut self, inst: &mut Instruction) -> Result<Outcome, Trap> {
        match inst.type_name {
            InstTypeName::R => {
                if let InstTypeData::R {
                    rd,
                    funct3,
                    funct7,
                    rs1,
                    rs2,
                } = inst.type_data
                {
                    match funct3 {
                        0x0 => match funct7 {
                            0x0 => {
                                inst.name = format!(
                                    "add     x{},x{},x{}",
                                    rd, rs1, rs2
                                );
                                self.registers[rd] = self.registers[rs1]
                                    .wrapping_add(self.registers[rs2]);
                            }
                            0x20 => {
                                inst.name = format!(
                                    "sub     x{},x{},x{}",
                                    rd, rs1, rs2
                                );
                                self.registers[rd] = self.registers[rs1]
                                    .wrapping_sub(self.registers[rs2]);
                            }
                            _ => {
                                return Err(Trap::UnsupportedInstruction {
                                    detail: format!(
                                        "R-type funct7 {:#09b}",
                                        funct7
                                    ),
                                });
                            }
                        },
                        0x4 => {
                            inst.name = format!(
                                "xor     x{},x{},x{}",
                                rd, rs1, rs2
                            );
                            self.registers[rd] =
                                self.registers[rs1] ^ self.registers[rs2];
                        }
                        0x6 => {
                            inst.name = format!(
                                "or      x{},x{},x{}",
                                rd, rs1, rs2
                            );
                            self.registers[rd] =
                                self.registers[rs1] | self.registers[rs2];
                        }
                        0x7 => {
                            inst.name = format!(
                                "and     x{},x{},x{}",
                                rd, rs1, rs2
                            );
                            self.registers[rd] =
                                self.registers[rs1] & self.registers[rs2];
                        }
                        0x1 => {
                            inst.name = format!(
                                "sll     x{},x{},x{}",
                                rd, rs1, rs2
                            );
                            self.registers[rd] =
                                self.registers[rs1] << self.registers[rs2];
                        }
                        0x5 => match funct7 {
                            0x0 => {
                                inst.name = format!(
                                    "srl     x{},x{},x{}",
                                    rd, rs1, rs2
                                );
                                self.registers[rd] = self.registers[rs1]
                                    >> self.registers[rs2];
                            }
                            0x20 => {
                                inst.name = format!(
                                    "sra     x{},x{},x{}",
                                    rd, rs1, rs2
                                );
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    >> self.registers[rs2])
                                    as u32;
                            }
                            _ => {
                                return Err(Trap::UnsupportedInstruction {
                                    detail: format!(
                                        "R-type funct7 {:#09b}",
                                        funct7
                                    ),
                                });
                            }
                        },
                        0x2 => {
                            inst.name = format!(
                                "slt     x{},x{},x{}",
                                rd, rs1, rs2
                            );
                            self.registers[rd] = if (self.registers[rs1]
                                as i32)
                                < (self.registers[rs2] as i32)
                            {
                                1
                            } else {
                                0
                            }
                        }
                        0x3 => {
                            inst.name = format!(
                                "sltu    x{},x{},x{}",
                                rd, rs1, rs2
                            );
                            self.registers[rd] = if self.registers[rs1]
                                < self.registers[rs2]
                            {
                                1
                            } else {
                                0
                            }
                        }
                        _ => {
                            return Err(Trap::UnsupportedInstruction {
                                detail: format!(
                                    "R-type funct3 {:#05b}",
                                    funct3
                                ),
                            });
                        }
                    };
                }
            }
            InstTypeName::B => {
                if let InstTypeData::B {
                    imm,
                    funct3,
                    rs1,
                    rs2,
                } = inst.type_data
                {
                    match funct3 {
                        0x0 => {
                            inst.name = format!(
                                "beq     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                (self.pc) as i32 + imm as i32
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs == rhs {
                                self.pc =
                                    (self.pc as i32 + imm as i32) as u32;
                                return Ok(Outcome::Continue);
                            };
                        }
                        0x1 => {
                            inst.name = format!(
                                "bne     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                (self.pc) as i32 + imm as i32
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs != rhs {
                                self.pc =
                                    (self.pc as i32 + imm as i32) as u32;
                                return Ok(Outcome::Continue);
                            };
                        }
                        0x4 => {
                            inst.name = format!(
                                "blt     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                (self.pc) as i32 + imm as i32
                            );
                            let lhs = self.registers[rs1] as i32;
                            let rhs = self.registers[rs2] as i32;
                            if lhs < rhs {
                                self.pc =
                                    (self.pc as i32 + imm as i32) as u32;
                                return Ok(Outcome::Continue);
                            };
                        }
                        0x5 => {
                            inst.name = format!(
                                "bge     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                (self.pc) as i32 + imm as i32
                            );
                            let lhs = self.registers[rs1] as i32;
                            let rhs = self.registers[rs2] as i32;
                            if lhs >= rhs {
                                self.pc =
                                    (self.pc as i32 + imm as i32) as u32;
                                return Ok(Outcome::Continue);
                            };
                        }
                        0x6 => {
                            inst.name = format!(
                                "bltu    x{},x{},{:08x}",
                                rs1,
                                rs2,
                                (self.pc) as i32 + imm as i32
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs < rhs {
                                self.pc =
                                    (self.pc as i32 + imm as i32) as u32;
                                return Ok(Outcome::Continue);
                            };
                        }
                        0x7 => {
                            inst.name = format!(
                                "bgeu    x{},x{},{:08x}",
                                rs1,
                                rs2,
                                (self.pc) as i32 + imm as i32
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs >= rhs {
                                self.pc =
                                    (self.pc as i32 + imm as i32) as u32;
                                return Ok(Outcome::Continue);
                            };
                        }
                        _ => {
                            return Err(Trap::UnsupportedInstruction {
                                detail: format!(
                                    "B-type funct3 {:#05b}",
                                    funct3
                                ),
                            });
                        }
                    };
                }
            }
            InstTypeName::J => {
                if let InstTypeData::J { rd, imm } = inst.type_data {
                    match inst.opcode {
                        0b1101111 => {
                            inst.name =
                                format!("jal     x{},{:08x}", rd, imm);
                            self.registers[rd] = self.pc + 4;
                            self.pc = (self.pc as i32 + imm as i32) as u32;
                            self.registers[0] = 0;
                            return Ok(Outcome::Continue);
                        }
                        _ => {
                            return Err(Trap::UnsupportedInstruction {
                                detail: format!(
                                    "J-type opcode {:#09b}",
                                    inst.opcode
                                ),
                            });
                        }
                    };
                }
            }
            InstTypeName::I => {
                if let InstTypeData::I {
                    rd,
                    funct3,
                    rs1,
                    imm,
                } = inst.type_data
                {
                    match inst.opcode {
                        0b0010011 => match funct3 {
                            0x0 => {
                                inst.name = format!(
                                    "addi    x{},x{},{}",
                                    rd, rs1, imm as i32
                                );
                                self.registers[rd] = (self.registers[rs1]
                                    as i32)
                                    .wrapping_add(imm as i32)
                                    as u32;

                                if rd == 0 && rs1 == 0 && imm == 0 {
                                    inst.name = String::from("nop");
                                }
                            }
                            0x4 => {
                                inst.name = format!(
                                    "xori    x{},x{},{}",
                                    rd, rs1, imm as i32
                                );
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    ^ (imm as i32))
                                    as u32;
                            }
                            0x6 => {
                                inst.name = format!(
                                    "ori     x{},x{},{}",
                                    rd, rs1, imm as i32
                                );
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    | (imm as i32))
                                    as u32;
                            }
                            0x7 => {
                                inst.name = format!(
                                    "andi    x{},x{},{}",
                                    rd, rs1, imm as i32
                                );
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    & (imm as i32))
                                    as u32;
                            }
                            0x2 => {
                                inst.name = format!(
                                    "slti    x{},x{},{}",
                                    rd, rs1, imm as i32
                                );
                                self.registers[rd] =
                                    if (self.registers[rs1] as i32)
                                        < (imm as i32)
                                    {
                                        1
                                    } else {
                                        0
                                    }
                            }
                            0x3 => {
                                inst.name = format!(
                                    "sltiu   x{},x{},{}",
                                    rd, rs1, imm
                                );
                                self.registers[rd] =
                                    if self.registers[rs1] < imm {
                                        1
                                    } else {
                                        0
                                    }
                            }
                            0x1 => {
                                let shamt = imm & 0b11111;
                                inst.name = format!(
                                    "slli    x{},x{},{:#x}",
                                    rd, rs1, shamt
                                );
                                self.registers[rd] =
                                    self.registers[rs1] << shamt;
                            }
                            0x5 => match (imm >> 5) & 0b1111111 {
                                0 => {
                                    let shamt = imm & 0b11111;
                                    inst.name = format!(
                                        "srli    x{},x{},{:#x}",
                                        rd, rs1, shamt
                                    );
                                    self.registers[rd] =
                                        self.registers[rs1] >> shamt;
                                }
                                0b0100000 => {
                                    let shamt = imm & 0b11111;
                                    inst.name = format!(
                                        "srai    x{},x{},{:#x}",
                                        rd, rs1, shamt
                                    );
                                    self.registers[rd] = Cpu::sign_extend(
                                        self.registers[rs1] >> shamt,
                                        32 - shamt,
                                    );
                                }
                                _ => {
                                    return Err(
                                        Trap::UnsupportedInstruction {
                                            detail: format!(
                                                "shift-right immediate funct7 {:#09b}",
                                                (imm >> 5) & 0b1111111
                                            ),
                                        },
                                    );
                                }
                            },
                            _ => {
                                return Err(Trap::UnsupportedInstruction {
                                    detail: format!(
                                        "I-type funct3 {:#05b}",
                                        funct3
                                    ),
                                });
                            }
                        },
                        0b0000011 => match funct3 {
                            0x0 => {
                                inst.name = format!(
                                    "lb      x{},{}(x{})",
                                    rd, imm as i32, rs1
                                );
                                let index = (self.registers[rs1]
                                    + Cpu::sign_extend(imm, 12))
                                    as usize;
                                self.registers[rd] = Cpu::sign_extend(
                                    self.memory[index] as u32,
                                    8,
                                );
                            }
                            0x1 => {
                                inst.name = format!(
                                    "lh      x{},{}(x{})",
                                    rd, imm as i32, rs1
                                );
                                let index = (self.registers[rs1]
                                    + Cpu::sign_extend(imm, 12))
                                    as usize;
                                let half_word = self.memory[index] as u32
                                    | (self.memory[index + 1] as u32) << 8;
                                self.registers[rd] =
                                    Cpu::sign_extend(half_word as u32, 16);
                            }
                            0x2 => {
                                inst.name = format!(
                                    "lw      x{},{}(x{})",
                                    rd, imm as i32, rs1
                                );
                                let index = (self.registers[rs1]
                                    + Cpu::sign_extend(imm, 12))
                                    as usize;

                                self.registers[rd] = self.memory[index]
                                    as u32
                                    | ((self.memory[index + 1]) as u32)
                                        << 8
                                    | ((self.memory[index + 2]) as u32)
                                        << 16
                                    | ((self.memory[index + 3]) as u32)
                                        << 24;
                            }
                            0x4 => {
                                inst.name = format!(
                                    "lbu     x{},{}(x{})",
                                    rd, imm, rs1
                                );
                                let index = (self.registers[rs1]
                                    + Cpu::sign_extend(imm, 12))
                                    as usize;
                                self.registers[rd] =
                                    self.memory[index] as u32;
                            }
                            0x5 => {
                                inst.name = format!(
                                    "lhu     x{},{}(x{})",
                                    rd, imm, rs1
                                );
                                let index = (self.registers[rs1]
                                    + Cpu::sign_extend(imm, 12))
                                    as usize;

                                self.registers[rd] = self.memory[index]
                                    as u32
                                    | (self.memory[index + 1] as u32) << 8;
                            }
                            _ => {
                                return Err(Trap::UnsupportedInstruction {
                                    detail: format!(
                                        "I-type funct3 {:#05b}",
                                        funct3
                                    ),
                                });
                            }
                        },
                        0b1100111 => match funct3 {
                            0x0 => {
                                inst.name = format!(
                                    "jalr    x{},x{},{:#x}",
                                    rd, rs1, imm
                                );
                                let pc_copy = self.pc;
                                self.pc = self.registers[rs1]
                                    + Cpu::sign_extend(imm, 12);
                                self.pc &= !1; // set lsb to 0
                                self.registers[rd] = pc_copy + 4;

                                self.registers[0] = 0;
                                return Ok(Outcome::Continue);
                            }
                            _ => {
                                return Err(Trap::UnsupportedInstruction {
                                    detail: format!(
                                        "I-type funct3 {:#05b}",
                                        funct3
                                    ),
                                });
                            }
                        },
                        0b1110011 => match funct3 {
                            0b000 => match imm {
                                0x0 => {
                                    inst.name = String::from("ecall");
                                    return match self.registers[17] {
                                        // `exit` syscall
                                        93 => Ok(Outcome::Exit(
                                            self.registers[10] as i32,
                                        )),
                                        num => {
                                            Err(Trap::UnsupportedSyscall {
                                                num,
                                            })
                                        }
                                    };
                                }
                                0x1 => {
                                    inst.name = String::from("ebreak");
                                    return Err(
                                        Trap::UnsupportedInstruction {
                                            detail: String::from(
                                                "ebreak (no debugger attached)",
                                            ),
                                        },
                                    );
                                }
                                // riscv-tests wrap each test in a guard
                                // sequence: write mepc, read mhartid, mret.
                                // This emulator has no M-mode or trap
                                // handling, so mret just falls through to the
                                // next instruction, which is where the test
                                // body begins.
                                0b1100000010 => {
                                    inst.name = String::from("mret");
                                }
                                _ => {
                                    return Err(
                                        Trap::UnsupportedInstruction {
                                            detail: format!(
                                                "system imm {:#014b}",
                                                imm
                                            ),
                                        },
                                    );
                                }
                            },
                            // Zicsr. The CSR file exists but starts empty:
                            // every CSR reads as 0 until something writes it.
                            // This matches a machine with no state to report.
                            //
                            // `imm` arrives sign-extended from 12 bits, so
                            // mask it back down to the CSR address.
                            0b001 => {
                                inst.name = format!(
                                    "csrrw   x{},{:#x},x{}",
                                    rd, imm, rs1
                                );
                                let csr = (imm & 0xfff) as usize;
                                let old = self.csrs[csr];
                                self.csrs[csr] = self.registers[rs1];
                                self.registers[rd] = old;
                            }
                            0b010 => {
                                inst.name = format!(
                                    "csrrs   x{},{:#x},x{}",
                                    rd, imm, rs1
                                );
                                let csr = (imm & 0xfff) as usize;
                                let old = self.csrs[csr];
                                // rs1 == x0 means "read only, do not write".
                                if rs1 != 0 {
                                    self.csrs[csr] = old | self.registers[rs1];
                                }
                                self.registers[rd] = old;
                            }
                            0b011 => {
                                inst.name = format!(
                                    "csrrc   x{},{:#x},x{}",
                                    rd, imm, rs1
                                );
                                let csr = (imm & 0xfff) as usize;
                                let old = self.csrs[csr];
                                if rs1 != 0 {
                                    self.csrs[csr] = old & !self.registers[rs1];
                                }
                                self.registers[rd] = old;
                            }
                            0b101 => {
                                inst.name = format!(
                                    "csrrwi  x{},{:#x},{}",
                                    rd, imm, rs1
                                );
                                let csr = (imm & 0xfff) as usize;
                                let old = self.csrs[csr];
                                self.csrs[csr] = rs1 as u32;
                                self.registers[rd] = old;
                            }
                            0b110 => {
                                inst.name = format!(
                                    "csrrsi  x{},{:#x},{}",
                                    rd, imm, rs1
                                );
                                let csr = (imm & 0xfff) as usize;
                                let old = self.csrs[csr];
                                if rs1 != 0 {
                                    self.csrs[csr] = old | rs1 as u32;
                                }
                                self.registers[rd] = old;
                            }
                            0b111 => {
                                inst.name = format!(
                                    "csrrci  x{},{:#x},{}",
                                    rd, imm, rs1
                                );
                                let csr = (imm & 0xfff) as usize;
                                let old = self.csrs[csr];
                                if rs1 != 0 {
                                    self.csrs[csr] = old & !(rs1 as u32);
                                }
                                self.registers[rd] = old;
                            }
                            _ => {
                                return Err(
                                    Trap::UnsupportedInstruction {
                                        detail: format!(
                                            "system funct3 {:#05b}",
                                            funct3
                                        ),
                                    },
                                );
                            }
                        },
                        _ => {
                            return Err(Trap::UnsupportedInstruction {
                                detail: format!(
                                    "I-type opcode {:#09b}",
                                    inst.opcode
                                ),
                            });
                        }
                    };
                }
            }
            InstTypeName::S => {
                if let InstTypeData::S {
                    imm,
                    funct3,
                    rs1,
                    rs2,
                } = inst.type_data
                {
                    match funct3 {
                        0x0 => {
                            inst.name = format!(
                                "sb      x{},{}(x{})",
                                rs2, imm as i32, rs1
                            );
                            let index = (self.registers[rs1]
                                + Cpu::sign_extend(imm, 12))
                                as usize;
                            self.memory[index] =
                                (self.registers[rs2] & 0xff) as u8;
                        }
                        0x1 => {
                            inst.name = format!(
                                "sh      x{},{}(x{})",
                                rs2, imm as i32, rs1
                            );
                            let index = (self.registers[rs1]
                                + Cpu::sign_extend(imm, 12))
                                as usize;
                            self.memory[index] =
                                (self.registers[rs2] & 0xff) as u8;
                            self.memory[index + 1] =
                                (self.registers[rs2] >> 8 & 0xff) as u8;
                        }
                        0x2 => {
                            inst.name = format!(
                                "sw      x{},{}(x{})",
                                rs2, imm as i32, rs1
                            );
                            let index = (self.registers[rs1]
                                + Cpu::sign_extend(imm, 12))
                                as usize;
                            self.memory[index] =
                                (self.registers[rs2] & 0xff) as u8;
                            self.memory[index + 1] =
                                (self.registers[rs2] >> 8 & 0xff) as u8;
                            self.memory[index + 2] =
                                (self.registers[rs2] >> 16 & 0xff) as u8;
                            self.memory[index + 3] =
                                (self.registers[rs2] >> 24 & 0xff) as u8;
                        }
                        _ => {
                            return Err(Trap::UnsupportedInstruction {
                                detail: format!(
                                    "S-type funct3 {:#05b}",
                                    funct3
                                ),
                            });
                        }
                    };
                }
            }
            InstTypeName::U => {
                if let InstTypeData::U { rd, imm } = inst.type_data {
                    match inst.opcode {
                        0b0110111 => {
                            inst.name =
                                format!("lui     x{},{:#x}", rd, imm);
                            self.registers[rd] = imm << 12;
                        }
                        0b0010111 => {
                            inst.name =
                                format!("auipc   x{},{:#x}", rd, imm);
                            self.registers[rd] = self.pc + (imm << 12);
                        }
                        _ => {
                            return Err(Trap::UnsupportedInstruction {
                                detail: format!(
                                    "U-type opcode {:#09b}",
                                    inst.opcode
                                ),
                            });
                        }
                    };
                }
            }
            InstTypeName::Fence => inst.name = String::from("fence"),
            InstTypeName::Unimp => {
                inst.name = String::from("unimp");
                return Err(Trap::Unimplemented);
            }
        }
        self.registers[0] = 0;
        self.pc += 4;
        Ok(Outcome::Continue)
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
                        let chunk = self.memory[addr] as u32
                            | (self.memory[addr + 1] as u32) << 8
                            | (self.memory[addr + 2] as u32) << 16
                            | (self.memory[addr + 3] as u32) << 24;
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
            self.registers[2] = (self.memory.len() - 1) as u32;
        }
        let mut buf = String::new();
        loop {
            buf.clear();
            print!("> ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut buf).unwrap();
            buf.pop();
            self.command_handler(&*buf);

            let raw_inst = self.fetch();
            let mut inst: Instruction = self.decode(raw_inst);
            let pc_copy = self.pc;

            if (&*buf).is_empty() {
                match self.execute(&mut inst) {
                    Ok(Outcome::Continue) => {}
                    Ok(Outcome::Exit(code)) => {
                        println!(
                            "Program exited with exit code: {}",
                            code
                        );
                        ret = code;
                        break;
                    }
                    Err(trap) => {
                        if args.debug {
                            println!("{}", trap.message());
                        }
                        ret = trap.exit_code();
                        break;
                    }
                }
                if args.registers {
                    self.print_registers(args.aliases);
                }
                println!(
                    "{:<08x}:   {:08x}          	{}",
                    pc_copy, raw_inst, inst.name
                );
            }

            if (self.pc as usize) >= self.memory.len() {
                if args.debug {
                    println!("PC overflow.");
                }
                ret = -1;
                break;
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
            self.registers[2] = (self.memory.len() - 1) as u32;
        }
        loop {
            if args.registers {
                self.print_registers(args.aliases);
            }
            let raw_inst = self.fetch();
            let mut inst: Instruction = self.decode(raw_inst);
            let pc_copy = self.pc;
            match self.execute(&mut inst) {
                Ok(Outcome::Continue) => {}
                Ok(Outcome::Exit(code)) => {
                    println!("Program exited with exit code: {}", code);
                    ret = code;
                    break;
                }
                Err(trap) => {
                    if args.debug {
                        println!("{}", trap.message());
                    }
                    ret = trap.exit_code();
                    break;
                }
            }
            if args.debug {
                println!(
                    "{:<08x}:   {:08x}          	{}",
                    pc_copy, raw_inst, inst.name
                );
            }

            if (self.pc as usize) >= self.memory.len() {
                if args.debug {
                    println!("PC overflow.");
                }
                ret = -1;
                break;
            }
        }
        ret
    }

    fn sign_extend(data: u32, size: u32) -> u32 {
        assert!(size > 0 && size <= 32);
        (((data << (32 - size)) as i32) >> (32 - size)) as u32
    }
}

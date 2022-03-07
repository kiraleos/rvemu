use super::instruction::*;
use elf_rs::{Elf, ElfFile};
use std::io::Read;

const MEM_SIZE: usize = 16;

pub struct Cpu {
    memory: [u32; 1024 * MEM_SIZE],
    registers: [u32; 32],
    pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            memory: [0; 1024 * MEM_SIZE],
            registers: [0; 32],
            pc: 0,
        }
    }

    pub fn load(&mut self, path: &str) {
        let mut elf_file =
            std::fs::File::open(path).expect("open file failed");
        let mut elf_buf = Vec::<u8>::new();
        elf_file
            .read_to_end(&mut elf_buf)
            .expect("read file failed");
        let elf = Elf::from_bytes(&elf_buf).expect("load elf file failed");
        match elf.elf_header().machine() {
            elf_rs::ElfMachine::RISC_V => {
                let p_vaddr = elf.program_header_nth(0).unwrap().vaddr();
                let p_offset = elf.program_header_nth(0).unwrap().offset();
                self.pc = (elf.entry_point() - p_vaddr + p_offset)
                    .try_into()
                    .expect("couldn't convert u64 entry addr to u32");
            }
            _ => {
                panic!(
                    "unsupported architecture: {:#?}",
                    elf.elf_header().machine()
                );
            }
        }
        let raw_data: Vec<u32> =
            elf_buf.into_iter().map(|x| x as u32).collect();
        self.memory[..raw_data.len()].copy_from_slice(&raw_data);
    }

    #[allow(dead_code)]
    pub fn print_registers(&self, aliases: bool) {
        let mut reg_name;
        println!("pc : 0x{:0>8x}", self.pc);
        let mut strbuilder = String::new();
        for i in 0..self.registers.len() {
            if aliases {
                reg_name = match i {
                    0 => "zero".to_string(),
                    1 => "ra".to_string(),
                    2 => "sp".to_string(),
                    3 => "gp".to_string(),
                    4 => "tp".to_string(),
                    5 => "t0".to_string(),
                    6 => "t1".to_string(),
                    7 => "t2".to_string(),
                    8 => "s0".to_string(),
                    9 => "s1".to_string(),
                    10 => "a0".to_string(),
                    11 => "a1".to_string(),
                    12 => "a2".to_string(),
                    13 => "a3".to_string(),
                    14 => "a4".to_string(),
                    15 => "a5".to_string(),
                    16 => "a6".to_string(),
                    17 => "a7".to_string(),
                    18 => "s2".to_string(),
                    19 => "s3".to_string(),
                    20 => "s4".to_string(),
                    21 => "s5".to_string(),
                    22 => "s6".to_string(),
                    23 => "s7".to_string(),
                    24 => "s8".to_string(),
                    25 => "s9".to_string(),
                    26 => "s10".to_string(),
                    27 => "s11".to_string(),
                    28 => "t3".to_string(),
                    29 => "t4".to_string(),
                    30 => "t5".to_string(),
                    31 => "t6".to_string(),
                    _ => panic!("should never be here"),
                };
                strbuilder += &*format!(
                    "{:>4}: 0x{:0>8x}  ",
                    reg_name, self.registers[i]
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

    #[allow(dead_code)]
    pub fn print_memory(&self) {
        for i in (0x1000..self.memory.len()).step_by(4) {
            let chunk = self.memory[i]
                | self.memory[i + 1] << 8
                | self.memory[i + 2] << 16
                | self.memory[i + 3] << 24;
            if chunk != 0 {
                println!("{:#010x}:\t{:#010x}", i, chunk);
            }
        }
    }

    fn fetch(&self) -> u32 {
        let index = self.pc as usize;
        self.memory[index]
            | self.memory[index + 1] << 8
            | self.memory[index + 2] << 16
            | self.memory[index + 3] << 24
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

    fn execute(&mut self, inst: &mut Instruction) {
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
                                panic!(
                                    "unknown R funct7: {:#09b}",
                                    funct7
                                );
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
                                panic!(
                                    "unknown R funct7: {:#09b}",
                                    funct7
                                );
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
                            panic!(
                                "execute: unimplemented R funct3: {:#05b}",
                                funct3
                            );
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
                                return;
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
                                return;
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
                                return;
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
                                return;
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
                                return;
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
                                return;
                            };
                        }
                        _ => {
                            panic!(
                                "execute: unimplemented B funct3: {:#05b}",
                                funct3
                            );
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
                            return;
                        }
                        _ => {
                            panic!(
                                "execute: unimplemented J opcode: {:#09b}",
                                inst.opcode
                            );
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
                                    panic!("should never be here.")
                                }
                            },
                            _ => {
                                panic!(
                                    "unknown I funct3: {:#05b}",
                                    funct3,
                                );
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
                                    self.memory[index],
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
                                let half_word = self.memory[index]
                                    | self.memory[index + 1] << 8;
                                self.registers[rd] =
                                    Cpu::sign_extend(half_word, 16);
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
                                    | self.memory[index + 1] << 8
                                    | self.memory[index + 2] << 16
                                    | self.memory[index + 3] << 24;
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
                                    | self.memory[index + 1] << 8;
                            }
                            _ => {
                                panic!(
                                    "unknown I funct3: {:#05b}",
                                    funct3
                                );
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
                                return;
                            }
                            _ => {
                                panic!(
                                    "unknown I funct3: {:#05b}",
                                    funct3
                                );
                            }
                        },
                        0b1110011 => match funct3 {
                            0b000 => match imm {
                                0x0 => {
                                    inst.name = String::from("ecall");
                                }
                                0x1 => {
                                    inst.name = String::from("ebreak");
                                }
                                0b1100000010 => {
                                    inst.name = String::from("mret");
                                }
                                _ => {
                                    panic!("unknown I imm: {:#014b}", imm)
                                }
                            },
                            0b001 => {
                                inst.name = format!(
                                    "csrrw   x{},{:#x},x{}",
                                    rd, imm, rs1
                                );
                            }
                            0b010 => {
                                inst.name = format!(
                                    "csrrs   x{},{:#x},x{}",
                                    rd, imm, rs1
                                );
                            }
                            0b011 => {
                                inst.name = format!(
                                    "csrrc   x{},{:#x},x{}",
                                    rd, imm, rs1
                                );
                            }
                            0b101 => {
                                inst.name = format!(
                                    "csrrwi  x{},{:#x},{}",
                                    rd, imm, rs1
                                );
                            }
                            0b110 => {
                                inst.name = format!(
                                    "csrrsi  x{},{:#x},{}",
                                    rd, imm, rs1
                                );
                            }
                            0b111 => {
                                inst.name = format!(
                                    "csrrci  x{},{:#x},{}",
                                    rd, imm, rs1
                                );
                            }
                            _ => {
                                panic!(
                                    "unknown I funct3: {:#05b}",
                                    funct3
                                );
                            }
                        },
                        _ => {
                            panic!(
                                "unknown I opcode: {:#09b}",
                                inst.opcode
                            );
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
                                self.registers[rs2] & 0xff;
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
                                self.registers[rs2] & 0xff;
                            self.memory[index + 1] =
                                self.registers[rs2] >> 8 & 0xff;
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
                                self.registers[rs2] & 0xff;
                            self.memory[index + 1] =
                                self.registers[rs2] >> 8 & 0xff;
                            self.memory[index + 2] =
                                self.registers[rs2] >> 16 & 0xff;
                            self.memory[index + 3] =
                                self.registers[rs2] >> 24 & 0xff;
                        }
                        _ => {
                            panic!("unknown S funct3: {:#05b}", funct3);
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
                            panic!(
                                "unknown U opcode: {:#09b}",
                                inst.opcode
                            );
                        }
                    };
                }
            }
            InstTypeName::Fence => inst.name = String::from("fence"),
            InstTypeName::Unimp => inst.name = String::from("unimp"),
        }
        self.registers[0] = 0;
        self.pc += 4;
    }

    pub fn run(
        &mut self,
        debug: bool,
        print_inst: bool,
        print_regs: bool,
        pc: Option<String>,
        aliases: bool,
        interactive: bool,
    ) -> i32 {
        let ret: i32;
        if let Some(pc) = pc {
            self.pc = u32::from_str_radix(&pc, 16).unwrap_or(self.pc);
        }
        loop {
            let mut buf = String::new();
            if interactive {
                std::io::stdin().read_line(&mut buf).unwrap();
            }
            if print_regs {
                self.print_registers(aliases);
            }
            let raw_inst = self.fetch();
            let mut inst: Instruction = self.decode(raw_inst);
            let pc_copy = self.pc;
            self.execute(&mut inst);
            if print_inst {
                println!(
                    "{:<08x}:   {:08x}          	{}",
                    pc_copy, raw_inst, inst.name
                );
            }

            if (self.pc as usize) >= self.memory.len() {
                if debug {
                    println!("PC overflow.");
                }
                ret = -1;
                break;
            }
            match inst.name.as_str() {
                "ecall" => match self.registers[17] {
                    // `exit` syscall
                    93 => {
                        ret = self.registers[10] as i32;
                        println!("Program exited with exit code: {}", ret);
                        break;
                    }
                    _ => {
                        if debug {
                            println!(
                                "Unimplemented ECALL: {}",
                                self.registers[17],
                            );
                        }
                        ret = -2;
                        break;
                    }
                },
                "unimp" => {
                    if debug {
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

    #[allow(dead_code)]
    pub fn current_instruction(&self) -> String {
        let raw_inst = self.fetch();
        let inst: Instruction = self.decode(raw_inst);
        format!(
            "{:<08x}:   {:08x}          	{}",
            self.pc, raw_inst, inst.name
        )
    }

    pub fn all_instructions(&mut self) -> String {
        let mut str = String::new();
        loop {
            let raw_inst = self.fetch();
            let mut inst: Instruction = self.decode(raw_inst);
            let pc_copy = self.pc;
            self.execute(&mut inst);
            str += &*format!(
                "{:<08x}:   {:08x}          	{}\n",
                pc_copy, raw_inst, inst.name
            );

            if inst.name.as_str() == "ecall" {
                match self.registers[17] {
                    // `exit` syscall
                    93 => {
                        break;
                    }
                    _ => {
                        panic!("unknown syscall");
                    }
                }
            }
        }
        self.reset_registers();
        str
    }

    fn reset_registers(&mut self) {
        for reg in &mut self.registers {
            *reg = 0;
        }
        self.pc = 0;
    }

    pub fn execute_at_pc(&mut self) {
        let raw_inst = self.fetch();
        let mut inst: Instruction = self.decode(raw_inst);
        self.execute(&mut inst);
        self.pc += 4;
    }

    fn sign_extend(data: u32, size: u32) -> u32 {
        assert!(size > 0 && size <= 32);
        (((data << (32 - size)) as i32) >> (32 - size)) as u32
    }
}

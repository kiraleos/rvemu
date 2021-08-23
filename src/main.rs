/*
    Fix imm values not being decoded as negative
*/

use object::{Object, ObjectSection};
const MEM_SIZE: usize = 32;

#[derive(Debug)]
struct Instruction {
    name: String,
    opcode: u32,
    type_data: InstTypeData,
    type_name: InstTypeName,
}

impl Instruction {
    fn new() -> Self {
        Instruction {
            name: String::from("default"),
            opcode: 0,
            type_data: InstTypeData::Unimp,
            type_name: InstTypeName::Unimp,
        }
    }
}

#[derive(Debug)]
enum InstTypeName {
    R,
    I,
    S,
    B,
    U,
    J,
    Unimp,
    Fence,
}

#[derive(Debug)]
enum InstTypeData {
    R {
        rd: usize,
        funct3: u32,
        rs1: usize,
        rs2: usize,
        funct7: u32,
    },
    I {
        rd: usize,
        funct3: u32,
        rs1: usize,
        imm: u32,
    },
    S {
        imm: u32,
        funct3: u32,
        rs1: usize,
        rs2: usize,
    },
    B {
        imm: u32,
        funct3: u32,
        rs1: usize,
        rs2: usize,
    },
    U {
        rd: usize,
        imm: u32,
    },
    J {
        rd: usize,
        imm: u32,
    },
    Unimp,
    Fence,
}

struct Cpu {
    memory: [u32; 1024 * MEM_SIZE],
    registers: [u32; 32],
    pc: u32,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            memory: [0; 1024 * MEM_SIZE],
            registers: [0; 32],
            pc: 0,
        }
    }

    fn load(&mut self, path: &str) {
        let file = std::fs::read(path).unwrap();
        let obj = object::File::parse(&*file).unwrap();
        let text_section =
            obj.section_by_name(".text.init").unwrap().data().unwrap();
        let text_section: Vec<u32> =
            text_section.iter().map(|x| *x as u32).collect();
        let text_section = text_section.as_slice();
        self.memory[..text_section.len()].copy_from_slice(text_section);

        self.pc = 0;
    }

    fn print_state(&self, aliases: bool) {
        let mut reg_name;
        for i in 0..self.registers.len() {
            if aliases {
                reg_name = match i {
                    0 => "x0".to_string(),
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
                    _ => "error".to_string(),
                };
                println!("{:>3}: 0x{:0>8x}", reg_name, self.registers[i]);
            } else {
                reg_name = i.to_string();
                println!("{:>2}: 0x{:0>8x}", reg_name, self.registers[i]);
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
                let imm = ((inst >> 20) & 0b111111111111) as i32 as u32;
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
                let imm = imm as i32 as u32;

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
                let imm = imm as i32 as u32;
                instruction.type_data = InstTypeData::J { rd, imm };
                instruction.type_name = InstTypeName::J;
            }

            // U type
            0b0110111 | 0b0010111 => {
                let rd = ((inst >> 7) & 0b11111) as usize;
                let imm = (inst >> 12) & 0b11111111111111111111;
                let imm = (imm << 12) as i32 as u32;
                instruction.type_data = InstTypeData::U { rd, imm };
                instruction.type_name = InstTypeName::U;
            }

            // Fence
            0b0001111 => {
                instruction.type_data = InstTypeData::Fence;
                instruction.type_name = InstTypeName::Fence;
            }

            _ => println!("decode: unimplemented opcode: {:#09b}", opcode),
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
                                inst.name = format!(
                                    "unimplemented R funct7: {:#09b}",
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
                                inst.name = format!(
                                    "unimplemented R funct7: {:#09b}",
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
                            inst.name = format!(
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
                                self.pc + imm
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs == rhs {
                                self.pc += imm;
                                return;
                            };
                        }
                        0x1 => {
                            inst.name = format!(
                                "bne     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                self.pc + imm
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs != rhs {
                                self.pc += imm;
                                return;
                            };
                        }
                        0x4 => {
                            inst.name = format!(
                                "blt     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                self.pc + imm
                            );
                            let lhs = self.registers[rs1] as i32;
                            let rhs = self.registers[rs2] as i32;
                            if lhs < rhs {
                                self.pc += imm;
                                return;
                            };
                        }
                        0x5 => {
                            inst.name = format!(
                                "bge     x{},x{},{:08x}",
                                rs1,
                                rs2,
                                self.pc + imm
                            );
                            let lhs = self.registers[rs1] as i32;
                            let rhs = self.registers[rs2] as i32;
                            if lhs >= rhs {
                                self.pc += imm;
                                return;
                            };
                        }
                        0x6 => {
                            inst.name = format!(
                                "bltu    x{},x{},{:08x}",
                                rs1,
                                rs2,
                                self.pc + imm
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs < rhs {
                                self.pc += imm;
                                return;
                            };
                        }
                        0x7 => {
                            inst.name = format!(
                                "bgeu    x{},x{},{:08x}",
                                rs1,
                                rs2,
                                self.pc + imm
                            );
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs >= rhs {
                                self.pc += imm;
                                return;
                            };
                        }
                        _ => {
                            inst.name = format!(
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
                            self.pc = self.pc.wrapping_add(imm);
                            self.registers[0] = 0;
                            return;
                        }
                        _ => {
                            inst.name = format!(
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
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    + (imm as i32))
                                    as u32;

                                if rd == 0 && rs1 == 0 && imm == 0 {
                                    inst.name = String::from("nop (addi)");
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
                                    "slli    x{},x{},x{}",
                                    rd, rs1, shamt
                                );
                                self.registers[rd] =
                                    self.registers[rs1] << shamt;
                            }
                            0x5 => match (imm >> 30) & 0b1 {
                                0 => {
                                    let shamt = imm & 0b11111;
                                    inst.name = format!(
                                        "srli    x{},x{},x{}",
                                        rd, rs1, shamt
                                    );
                                    self.registers[rd] =
                                        self.registers[rs1] >> shamt;
                                }
                                1 => {
                                    let shamt = imm & 0b11111;
                                    inst.name = format!(
                                        "srai    x{},x{},x{}",
                                        rd, rs1, shamt
                                    );
                                    self.registers[rd] =
                                        (self.registers[rs1] as i32
                                            >> shamt)
                                            as u32;
                                }
                                _ => {
                                    inst.name =
                                        format!("should never be here.");
                                }
                            },
                            _ => {
                                inst.name = format!(
                                "execute: unimplemented I funct3: {:#05b}",
                                funct3
                            );
                            }
                        },
                        0b0000011 => match funct3 {
                            0x0 => {
                                inst.name = format!(
                                    "lb      x{},x{},{:#x}",
                                    rd, rs1, imm
                                );
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    ((self.memory[index] & 0xff) as i32)
                                        as u32;
                            }
                            0x1 => {
                                inst.name = format!(
                                    "lh      x{},x{},{:#x}",
                                    rd, rs1, imm
                                );
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    ((self.memory[index] & 0xffff) as i32)
                                        as u32;
                            }
                            0x2 => {
                                inst.name = format!(
                                    "lw      x{},x{},{:#x}",
                                    rd, rs1, imm
                                );
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] = self.memory[index];
                            }
                            0x4 => {
                                inst.name = format!(
                                    "lbu     x{},x{},{:#x}",
                                    rd, rs1, imm
                                );
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    self.memory[index] & 0xff;
                            }
                            0x5 => {
                                inst.name = format!(
                                    "lhu     x{},x{},{:#x}",
                                    rd, rs1, imm
                                );
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    self.memory[index] & 0xffff;
                            }
                            _ => {
                                inst.name = format!(
                                    "execute: unimplemented I funct3: {:#05b}",
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
                                self.registers[rd] = self.pc + 4;
                                self.pc =
                                    imm.wrapping_add(self.registers[rs1]);
                                self.registers[0] = 0;
                                return;
                            }
                            _ => {
                                inst.name = format!(
                                    "execute: unimplemented I funct3: {:#05b}",
                                    funct3
                                );
                            }
                        },
                        0b1110011 => {
                            match funct3 {
                                0b000 => match imm {
                                    0x0 => {
                                        inst.name = String::from("ecall");
                                    }
                                    0x1 => {
                                        inst.name = String::from("ebreak");
                                    }
                                    _ => {
                                        inst.name = format!("unknown ecall/ebreak imm: {:#014b}", imm);
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
                                    inst.name = format!("unknown ecall/ebreak funct3: {:#05b}", funct3);
                                }
                            }
                        }
                        _ => {
                            inst.name = format!(
                                "execute: unimplemented I opcode: {:#09b}",
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
                                "sb      x{},x{},{:#x}",
                                rs1, rs2, imm
                            );
                            let index = self.registers[rs1]
                                .wrapping_add(imm)
                                as usize;
                            self.memory[index] = self.memory[index]
                                | self.registers[rs2] & 0xff;
                        }
                        0x1 => {
                            inst.name = format!(
                                "sh      x{},x{},{:#x}",
                                rs1, rs2, imm
                            );
                            let index = self.registers[rs1]
                                .wrapping_add(imm)
                                as usize;
                            self.memory[index] = self.memory[index]
                                | self.registers[rs2] & 0xffff;
                        }
                        0x2 => {
                            inst.name = format!(
                                "sw      x{},x{},{:#x}",
                                rs1, rs2, imm
                            );
                            let index = self.registers[rs1]
                                .wrapping_add(imm)
                                as usize;
                            self.memory[index] =
                                self.registers[rs2] & 0xffffffff;
                        }
                        _ => {
                            inst.name = format!(
                                "execute: unimplemented S funct3: {:#05b}",
                                funct3
                            );
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
                            self.registers[rd] = imm;
                        }
                        0b0010111 => {
                            inst.name =
                                format!("auipc   x{},{:#x}", rd, imm);
                            self.registers[rd] = self.pc.wrapping_add(imm);
                        }
                        _ => {
                            inst.name = format!(
                                "execute: unimplemented U opcode: {:#09b}",
                                inst.opcode
                            );
                        }
                    };
                }
            }
            InstTypeName::Fence => inst.name = String::from("nop (fence)"),
            InstTypeName::Unimp => inst.name = String::from("unimp"),
        }
        self.registers[0] = 0;
        self.pc += 4;
    }

    fn run(&mut self, debug: bool) -> i32 {
        if debug {
            println!("PC              RAW_INST                INST");
        }
        let mut ret: i32 = -1;
        loop {
            let raw_inst = self.fetch();
            let mut inst: Instruction = self.decode(raw_inst);
            let pc_copy = self.pc;
            self.execute(&mut inst);
            if debug {
                println!(
                    "{:<08x}:       {:08x}                {}",
                    pc_copy, raw_inst, inst.name
                );
            }

            if (self.pc as usize) > self.memory.len() {
                if debug {
                    println!("PC overflow.");
                }
                break;
            }
            match inst.type_name {
                InstTypeName::Unimp => {
                    if debug {
                        println!("Reached an `unimp` instruction.");
                    }
                    break;
                }
                _ => {}
            }
            match inst.name.as_str() {
                "ecall" => match self.registers[17] {
                    93 => {
                        if debug {
                            println!(
                                "Program exited with status code: {}",
                                self.registers[10]
                            );
                        }
                        ret = self.registers[10] as i32;
                        break;
                    }
                    _ => {
                        if debug {
                            println!(
                                "Unimplemented ECALL: {}",
                                self.registers[17],
                            );
                        }
                        break;
                    }
                },
                _ => {}
            }
        }
        ret
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| "./tests/addi".into());
    let mut cpu = Cpu::new();
    cpu.load(&path);
    let ret = cpu.run(false);
    if ret != 0 {
        cpu.run(true);
    }
}

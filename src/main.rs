use object::{Object, ObjectSection};
const MEM_SIZE: usize = 32;

#[derive(Debug)]
struct Instruction {
    name: &'static str,
    opcode: u32,
    type_data: InstTypeData,
    type_name: InstTypeName,
}

impl Instruction {
    fn new() -> Self {
        Instruction {
            name: "default",
            opcode: 0,
            type_data: InstTypeData::Unimplemented,
            type_name: InstTypeName::Unimplemented,
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
    Unimplemented,
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
    Unimplemented,
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
                                inst.name = "add";
                                self.registers[rd] = self.registers[rs1]
                                    .wrapping_add(self.registers[rs2]);
                            }
                            0x20 => {
                                inst.name = "sub";
                                self.registers[rd] = self.registers[rs1]
                                    .wrapping_sub(self.registers[rs2]);
                            }
                            _ => {
                                println!(
                                    "unimplemented R funct7: {:#09b}",
                                    funct7
                                );
                            }
                        },
                        0x4 => {
                            inst.name = "xor";
                            self.registers[rd] =
                                self.registers[rs1] ^ self.registers[rs2];
                        }
                        0x6 => {
                            inst.name = "or";
                            self.registers[rd] =
                                self.registers[rs1] | self.registers[rs2];
                        }
                        0x7 => {
                            inst.name = "and";
                            self.registers[rd] =
                                self.registers[rs1] & self.registers[rs2];
                        }
                        0x1 => {
                            inst.name = "sll";
                            self.registers[rd] =
                                self.registers[rs1] << self.registers[rs2];
                        }
                        0x5 => match funct7 {
                            0x0 => {
                                inst.name = "srl";
                                self.registers[rd] = self.registers[rs1]
                                    >> self.registers[rs2];
                            }
                            0x20 => {
                                inst.name = "sra";
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    >> self.registers[rs2])
                                    as u32;
                            }
                            _ => {
                                println!(
                                    "unimplemented R funct7: {:#09b}",
                                    funct7
                                );
                            }
                        },
                        0x2 => {
                            inst.name = "slt";
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
                            inst.name = "sltu";
                            self.registers[rd] = if self.registers[rs1]
                                < self.registers[rs2]
                            {
                                1
                            } else {
                                0
                            }
                        }
                        _ => {
                            println!(
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
                            inst.name = "beq";
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs == rhs {
                                self.pc += imm
                            };
                        }
                        0x1 => {
                            inst.name = "bne";
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs != rhs {
                                self.pc += imm
                            };
                        }
                        0x4 => {
                            inst.name = "blt";
                            let lhs = self.registers[rs1] as i32;
                            let rhs = self.registers[rs2] as i32;
                            if lhs < rhs {
                                self.pc += imm
                            };
                        }
                        0x5 => {
                            inst.name = "bge";
                            let lhs = self.registers[rs1] as i32;
                            let rhs = self.registers[rs2] as i32;
                            if lhs >= rhs {
                                self.pc += imm
                            };
                        }
                        0x6 => {
                            inst.name = "bltu";
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs < rhs {
                                self.pc += imm
                            };
                        }
                        0x7 => {
                            inst.name = "bgeu";
                            let lhs = self.registers[rs1];
                            let rhs = self.registers[rs2];
                            if lhs >= rhs {
                                self.pc += imm
                            };
                        }
                        _ => {
                            println!(
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
                            inst.name = "jal";
                            self.registers[rd] = self.pc + 4;
                            self.pc = self.pc.wrapping_add(imm);
                        }
                        _ => {
                            println!(
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
                                inst.name = "addi";
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    + (imm as i32))
                                    as u32;

                                if rd == 0 && rs1 == 0 && imm == 0 {
                                    inst.name = "nop (addi)";
                                }
                            }
                            0x4 => {
                                inst.name = "xori";
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    ^ (imm as i32))
                                    as u32;
                            }
                            0x6 => {
                                inst.name = "ori";
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    | (imm as i32))
                                    as u32;
                            }
                            0x7 => {
                                inst.name = "andi";
                                self.registers[rd] = ((self.registers[rs1]
                                    as i32)
                                    & (imm as i32))
                                    as u32;
                            }
                            0x2 => {
                                inst.name = "slti";
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
                                inst.name = "sltiu";
                                self.registers[rd] =
                                    if self.registers[rs1] < imm {
                                        1
                                    } else {
                                        0
                                    }
                            }
                            0x1 => {
                                inst.name = "slli";
                                let shamt = imm & 0b11111;
                                self.registers[rd] =
                                    self.registers[rs1] << shamt;
                            }
                            0x5 => match (imm >> 30) & 0b1 {
                                0 => {
                                    inst.name = "srli";
                                    let shamt = imm & 0b11111;
                                    self.registers[rd] =
                                        self.registers[rs1] >> shamt;
                                }
                                1 => {
                                    inst.name = "srai";
                                    let shamt = imm & 0b11111;
                                    self.registers[rd] =
                                        (self.registers[rs1] as i32
                                            >> shamt)
                                            as u32;
                                }
                                _ => {
                                    println!("should never be here.");
                                }
                            },
                            _ => {
                                println!(
                                "execute: unimplemented I funct3: {:#05b}",
                                funct3
                            );
                            }
                        },
                        0b0000011 => match funct3 {
                            0x0 => {
                                inst.name = "lb";
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    ((self.memory[index] & 0xff) as i32)
                                        as u32;
                            }
                            0x1 => {
                                inst.name = "lh";
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    ((self.memory[index] & 0xffff) as i32)
                                        as u32;
                            }
                            0x2 => {
                                inst.name = "lw";
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] = self.memory[index];
                            }
                            0x4 => {
                                inst.name = "lbu";
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    self.memory[index] & 0xff;
                            }
                            0x5 => {
                                inst.name = "lhu";
                                let index =
                                    (self.registers[rs1] + imm) as usize;
                                self.registers[rd] =
                                    self.memory[index] & 0xffff;
                            }
                            _ => {
                                println!(
                                    "execute: unimplemented I funct3: {:#05b}",
                                    funct3
                                );
                            }
                        },
                        0b1100111 => match funct3 {
                            0x0 => {
                                inst.name = "jalr";
                                self.registers[rd] = self.pc + 4;
                                self.pc =
                                    imm.wrapping_add(self.registers[rs1]);
                            }
                            _ => {
                                println!(
                                    "execute: unimplemented I funct3: {:#05b}",
                                    funct3
                                );
                            }
                        },
                        0b1110011 => match funct3 {
                            0x0 => match imm {
                                0x0 => {
                                    inst.name = "ecall";
                                }
                                0x1 => {
                                    inst.name = "ebreak";
                                }
                                _ => {
                                    dbg!("shouldnt be here");
                                }
                            },
                            _ => {
                                dbg!("shouldnt be here");
                            }
                        },
                        _ => {
                            println!(
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
                            inst.name = "sb";
                            let index = self.registers[rs1]
                                .wrapping_add(imm)
                                as usize;
                            self.memory[index] = self.memory[index]
                                | self.registers[rs2] & 0xff;
                        }
                        0x1 => {
                            inst.name = "sh";
                            let index = self.registers[rs1]
                                .wrapping_add(imm)
                                as usize;
                            self.memory[index] = self.memory[index]
                                | self.registers[rs2] & 0xffff;
                        }
                        0x2 => {
                            inst.name = "sw";
                            let index = self.registers[rs1]
                                .wrapping_add(imm)
                                as usize;
                            self.memory[index] =
                                self.registers[rs2] & 0xffffffff;
                        }
                        _ => {
                            println!(
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
                            inst.name = "lui";
                            self.registers[rd] = imm;
                        }
                        0b0010111 => {
                            inst.name = "auipc";
                            self.registers[rd] = self.pc.wrapping_add(imm);
                        }
                        _ => {
                            println!(
                                "execute: unimplemented U opcode: {:#09b}",
                                inst.opcode
                            );
                        }
                    };
                }
            }
            InstTypeName::Fence => inst.name = "nop (fence)",
            InstTypeName::Unimplemented => {
                println!(
                    "execute: unimplemented instruction type: {:#?}",
                    inst
                );
            }
        }
    }

    fn run(&mut self) -> usize {
        let mut cycles = 0usize;
        loop {
            let inst = self.fetch();
            let opcode = inst & 0b1111111;
            if (self.pc as usize) >= self.memory.len() || opcode == 0 {
                return cycles;
            }
            let mut inst: Instruction = self.decode(inst);
            self.execute(&mut inst);
            println!("{:2}: {:#}", cycles + 1, inst.name);
            self.pc = self.pc + 4;
            cycles += 1;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: riscv-emulator <file>");
        return;
    }
    let mut cpu = Cpu::new();
    cpu.load(args[1].as_str());
    cpu.run();
    println!("{:#x?}\npc: {:#x}", cpu.registers, cpu.pc);
}

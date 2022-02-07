#[derive(Debug)]
pub struct Instruction {
    pub name: String,
    pub opcode: u32,
    pub type_data: InstTypeData,
    pub type_name: InstTypeName,
}

impl Instruction {
    pub fn new() -> Self {
        Instruction {
            name: String::from("default"),
            opcode: 0,
            type_data: InstTypeData::Unimp,
            type_name: InstTypeName::Unimp,
        }
    }
}

#[derive(Debug)]
pub enum InstTypeName {
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
pub enum InstTypeData {
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

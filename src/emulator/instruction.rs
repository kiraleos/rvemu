//! Decoding of RV32I instruction words.
//!
//! [`Instruction::decode`] is a pure function from a 32-bit word to the fields
//! that the instruction's format defines. Executing a decoded instruction is
//! the job of [`crate::emulator::cpu`].

/// A decoded RV32I instruction: exactly the fields its format defines.
///
/// There is one variant per opcode rather than a separate "format" tag and a
/// bag of fields, so an `Instruction` is always self-consistent and every
/// dispatch over it can be exhaustive.
///
/// Immediates are stored as `u32` bit patterns. The three formats that carry a
/// signed immediate (`OpImm`, `Load`, `Store`, `Branch`, `Jump`,
/// `JumpRegister`) have already had it sign extended by [`sign_extend`] to the
/// full 32 bits, so `imm as i32` recovers the original value and
/// `pc.wrapping_add(imm)` computes a target address directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// `OP` (0b011_0011): register-register arithmetic and logic.
    R {
        rd: usize,
        funct3: u32,
        rs1: usize,
        rs2: usize,
        funct7: u32,
    },
    /// `OP-IMM` (0b001_0011): register-immediate arithmetic and logic.
    OpImm {
        rd: usize,
        funct3: u32,
        rs1: usize,
        imm: u32,
    },
    /// `LOAD` (0b000_0011): load from memory.
    Load {
        rd: usize,
        funct3: u32,
        rs1: usize,
        imm: u32,
    },
    /// `STORE` (0b010_0011): store to memory.
    Store {
        imm: u32,
        funct3: u32,
        rs1: usize,
        rs2: usize,
    },
    /// `BRANCH` (0b110_0011): conditional branch.
    Branch {
        imm: u32,
        funct3: u32,
        rs1: usize,
        rs2: usize,
    },
    /// `JAL` (0b110_1111): jump and link.
    Jump { rd: usize, imm: u32 },
    /// `JALR` (0b110_0111): jump and link register.
    JumpRegister { rd: usize, rs1: usize, imm: u32 },
    /// `LUI` (0b011_0111): load the upper 20 bits of an immediate.
    Lui { rd: usize, imm: u32 },
    /// `AUIPC` (0b001_0111): add the upper 20 bits of an immediate to the pc.
    Auipc { rd: usize, imm: u32 },
    /// `SYSTEM` (0b111_0011): `ecall`, `ebreak`, `mret` and the CSR
    /// instructions. This emulator has no CSRs, so the CSR forms decode but
    /// do nothing.
    System {
        rd: usize,
        funct3: u32,
        rs1: usize,
        imm: u32,
    },
    /// `MISC-MEM` (0b000_1111): `fence` and `fence.i`. Both are no-ops on a
    /// single hart with no devices to order against.
    Fence,
    /// Not a valid RV32I encoding, or one this emulator refuses to run.
    /// The emulator stops when it reaches one of these.
    Unsupported,
}

/// Field positions within an instruction word, per the RV32I spec:
///
/// ```text
///   31        25 24     20 19     15 14  12 11      7 6        0
///  ┌───────────┬─────────┬─────────┬─────┬─────────┬──────────┐
///  │  funct7   │   rs2   │   rs1   │ft3  │   rd    │  opcode  │
///  └───────────┴─────────┴─────────┴─────┴─────────┴──────────┘
/// ```
/// The B, J, S and U formats scatter their immediates across those fields.
mod opcode {
    pub const LOAD: u32 = 0b000_0011;
    pub const MISC_MEM: u32 = 0b000_1111;
    pub const OP_IMM: u32 = 0b001_0011;
    pub const AUIPC: u32 = 0b001_0111;
    pub const STORE: u32 = 0b010_0011;
    pub const OP: u32 = 0b011_0011;
    pub const LUI: u32 = 0b011_0111;
    pub const BRANCH: u32 = 0b110_0011;
    pub const JALR: u32 = 0b110_0111;
    pub const JAL: u32 = 0b110_1111;
    pub const SYSTEM: u32 = 0b111_0011;
}

/// `wfi` ("wait for interrupt") is encoded as `csrrw x0, 0xc00, x0`.
///
/// Nothing in this emulator can ever wake the hart up, so the instruction is
/// reported as unsupported instead of spinning forever.
const WFI: u32 = 0xc00_1073;

impl Instruction {
    /// Decodes a raw 32-bit instruction word.
    ///
    /// Words that are not RV32I encodings decode to [`Instruction::Unsupported`].
    /// Field values that are individually invalid (an unknown `funct3`, say)
    /// are *not* rejected here; they are reported when the instruction is
    /// executed, which keeps this function a plain description of the encoding.
    pub fn decode(inst: u32) -> Self {
        if inst == 0 || inst == WFI {
            return Instruction::Unsupported;
        }

        match self::opcode(inst) {
            opcode::OP => Instruction::R {
                rd: rd(inst),
                funct3: funct3(inst),
                rs1: rs1(inst),
                rs2: rs2(inst),
                funct7: funct7(inst),
            },
            opcode::OP_IMM => Instruction::OpImm {
                rd: rd(inst),
                funct3: funct3(inst),
                rs1: rs1(inst),
                imm: sign_extend(field(inst, 20, 12), 12),
            },
            opcode::LOAD => Instruction::Load {
                rd: rd(inst),
                funct3: funct3(inst),
                rs1: rs1(inst),
                imm: sign_extend(field(inst, 20, 12), 12),
            },
            opcode::JALR => Instruction::JumpRegister {
                rd: rd(inst),
                rs1: rs1(inst),
                imm: sign_extend(field(inst, 20, 12), 12),
            },
            opcode::SYSTEM => Instruction::System {
                rd: rd(inst),
                funct3: funct3(inst),
                rs1: rs1(inst),
                imm: sign_extend(field(inst, 20, 12), 12),
            },
            opcode::STORE => Instruction::Store {
                // imm[11:5] in inst[31:25], imm[4:0] in inst[11:7].
                imm: sign_extend(field(inst, 25, 7) << 5 | field(inst, 7, 5), 12),
                funct3: funct3(inst),
                rs1: rs1(inst),
                rs2: rs2(inst),
            },
            opcode::BRANCH => Instruction::Branch {
                // imm[12|10:5|4:1|11] in inst[31|30:25|11:8|7].
                //
                // The assembled immediate occupies bits 12..1 but is only 13
                // bits wide, so it is sign extended from bit 12. Sign extending
                // one bit too few silently misplaces every offset from 2048 up
                // to 4094, and every offset from -4096 to -2049.
                imm: sign_extend(
                    field(inst, 31, 1) << 12
                        | field(inst, 7, 1) << 11
                        | field(inst, 25, 6) << 5
                        | field(inst, 8, 4) << 1,
                    13,
                ),
                funct3: funct3(inst),
                rs1: rs1(inst),
                rs2: rs2(inst),
            },
            opcode::JAL => Instruction::Jump {
                rd: rd(inst),
                // imm[20|19:12|11|10:1] in inst[31|19:12|20|30:21], 21 bits
                // wide, so sign extended from bit 20.
                imm: sign_extend(
                    field(inst, 31, 1) << 20
                        | field(inst, 12, 8) << 12
                        | field(inst, 20, 1) << 11
                        | field(inst, 21, 10) << 1,
                    21,
                ),
            },
            opcode::LUI => Instruction::Lui {
                rd: rd(inst),
                imm: field(inst, 12, 20),
            },
            opcode::AUIPC => Instruction::Auipc {
                rd: rd(inst),
                imm: field(inst, 12, 20),
            },
            opcode::MISC_MEM => Instruction::Fence,
            _ => Instruction::Unsupported,
        }
    }
}

/// Sign extends the low `bits` bits of `value` to a full 32-bit pattern.
///
/// `bits` is the width of the *encoded* field, which is not the same as the
/// width of the immediate it holds: a B-type immediate is assembled into
/// bits 12..1 but is only 13 bits wide, while the bits in between are zero.
pub fn sign_extend(value: u32, bits: u32) -> u32 {
    assert!(bits > 0 && bits <= 32, "bit width out of range: {bits}");
    (((value << (32 - bits)) as i32) >> (32 - bits)) as u32
}

/// Extracts the `width`-bit field of `inst` that starts at bit `shift`.
///
/// Every scattered immediate bit is read through this, so a field can never
/// pick up a neighbouring one by accident.
fn field(inst: u32, shift: u32, width: u32) -> u32 {
    (inst >> shift) & ((1 << width) - 1)
}

fn opcode(inst: u32) -> u32 {
    field(inst, 0, 7)
}

fn rd(inst: u32) -> usize {
    field(inst, 7, 5) as usize
}

fn funct3(inst: u32) -> u32 {
    field(inst, 12, 3)
}

fn rs1(inst: u32) -> usize {
    field(inst, 15, 5) as usize
}

fn rs2(inst: u32) -> usize {
    field(inst, 20, 5) as usize
}

fn funct7(inst: u32) -> u32 {
    field(inst, 25, 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an instruction word from its bit fields, for readable tables.
    struct Enc;

    impl Enc {
        const fn r(opcode: u32, rd: u32, funct3: u32, rs1: u32, rs2: u32, funct7: u32) -> u32 {
            (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
        }
        const fn i(opcode: u32, rd: u32, funct3: u32, rs1: u32, imm: u32) -> u32 {
            (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
        }
        const fn s(opcode: u32, funct3: u32, rs1: u32, rs2: u32, imm: u32) -> u32 {
            ((imm >> 5) << 25)
                | (rs2 << 20)
                | (rs1 << 15)
                | (funct3 << 12)
                | ((imm & 0x1f) << 7)
                | opcode
        }
        const fn b(opcode: u32, funct3: u32, rs1: u32, rs2: u32, imm: i32) -> u32 {
            // A 13-bit signed even offset, so -4096..=4094.
            assert!(imm % 2 == 0 && -4096 <= imm && imm <= 4094);
            let imm = imm as u32;
            ((imm >> 12) & 1) << 31
                | ((imm >> 5) & 0b11_1111) << 25
                | (rs2 << 20)
                | (rs1 << 15)
                | (funct3 << 12)
                | ((imm >> 1) & 0b1111) << 8
                | ((imm >> 11) & 1) << 7
                | opcode
        }
        const fn u(opcode: u32, rd: u32, imm: u32) -> u32 {
            (imm << 12) | (rd << 7) | opcode
        }
        const fn j(opcode: u32, rd: u32, imm: i32) -> u32 {
            // A 21-bit signed even offset, so -1048576..=1048574.
            assert!(imm % 2 == 0 && -1_048_576 <= imm && imm <= 1_048_574);
            let imm = imm as u32;
            ((imm >> 20) & 1) << 31
                | ((imm >> 1) & 0b11_1111_1111) << 21
                | ((imm >> 11) & 1) << 20
                | ((imm >> 12) & 0xff) << 12
                | (rd << 7)
                | opcode
        }
    }

    #[test]
    fn r_type_fields() {
        assert_eq!(
            Instruction::decode(Enc::r(0b011_0011, 3, 0b000, 1, 2, 0b000_0000)),
            Instruction::R {
                rd: 3,
                funct3: 0,
                rs1: 1,
                rs2: 2,
                funct7: 0
            }
        );
        assert_eq!(
            Instruction::decode(Enc::r(0b011_0011, 31, 0b111, 0, 0, 0b010_0000)),
            Instruction::R {
                rd: 31,
                funct3: 7,
                rs1: 0,
                rs2: 0,
                funct7: 0b010_0000
            }
        );
    }

    #[test]
    fn i_type_immediate_is_sign_extended() {
        // addi x1, x0, -1
        let inst = Enc::i(0b001_0011, 1, 0b000, 0, -1i32 as u32);
        assert_eq!(
            Instruction::decode(inst),
            Instruction::OpImm {
                rd: 1,
                funct3: 0,
                rs1: 0,
                imm: 0xffff_ffff
            }
        );
        // The most negative 12-bit immediate must not wrap to a positive.
        assert_eq!(
            Instruction::decode(Enc::i(0b001_0011, 1, 0b000, 0, 0x800)),
            Instruction::OpImm {
                rd: 1,
                funct3: 0,
                rs1: 0,
                imm: 0xffff_f800
            }
        );
        // ...and the most positive one must not sign extend.
        assert_eq!(
            Instruction::decode(Enc::i(0b001_0011, 1, 0b000, 0, 0x7ff)),
            Instruction::OpImm {
                rd: 1,
                funct3: 0,
                rs1: 0,
                imm: 0x7ff
            }
        );
    }

    #[test]
    fn s_type_immediate() {
        // sw x2, -4(x1)
        assert_eq!(
            Instruction::decode(Enc::s(0b010_0011, 0b010, 1, 2, -4i32 as u32)),
            Instruction::Store {
                imm: 0xffff_fffc,
                funct3: 2,
                rs1: 1,
                rs2: 2
            }
        );
        // sb x2, 2047(x1): the largest positive S-type immediate.
        assert_eq!(
            Instruction::decode(Enc::s(0b010_0011, 0b000, 1, 2, 2047)),
            Instruction::Store {
                imm: 2047,
                funct3: 0,
                rs1: 1,
                rs2: 2
            }
        );
    }

    #[test]
    fn b_type_immediate() {
        // beq x1, x2, -8
        assert_eq!(
            Instruction::decode(Enc::b(0b110_0011, 0b000, 1, 2, -8)),
            Instruction::Branch {
                imm: 0xffff_fff8,
                funct3: 0,
                rs1: 1,
                rs2: 2
            }
        );
    }

    /// The immediate carried by a decoded branch or jump.
    fn imm_of(inst: Instruction) -> u32 {
        match inst {
            Instruction::Branch { imm, .. } | Instruction::Jump { imm, .. } => imm,
            other => panic!("expected a branch or a jump, got {other:?}"),
        }
    }

    /// B-type offsets that straddle the point where the encoding's immediate
    /// bits stop agreeing with each other: from 2048 upwards `imm[11]` and
    /// `imm[12]` disagree. All of them are even, because the format has no
    /// `imm[0]` and so cannot encode an odd offset at all.
    const BRANCH_OFFSETS: [i32; 9] = [0, 4, -4, 0x7fc, -0x800, 0x800, -0x802, 0xffe, -0x1000];

    /// J-type offsets, where the same disagreement is between `imm[11]` and
    /// `imm[20]` and the range is twenty bits wider.
    const JUMP_OFFSETS: [i32; 13] = [
        0, 4, -4, 0x7fc, -0x800, 0x800, -0x802, 0xffe, -0x1000, 0x1000, -0x100_000, 0xff_ffe,
        -0x1_0000,
    ];

    #[test]
    fn branch_immediate_survives_decoding() {
        for offset in BRANCH_OFFSETS {
            assert_eq!(
                imm_of(Instruction::decode(Enc::b(0b110_0011, 0b000, 1, 2, offset))),
                offset as u32,
                "branch offset {offset:#x}"
            );
        }
    }

    #[test]
    fn jump_immediate_survives_decoding() {
        for offset in JUMP_OFFSETS {
            assert_eq!(
                imm_of(Instruction::decode(Enc::j(0b110_1111, 1, offset))),
                offset as u32,
                "jump offset {offset:#x}"
            );
        }
    }

    #[test]
    fn u_type_immediate() {
        // lui x5, 0xabcde
        assert_eq!(
            Instruction::decode(Enc::u(0b011_0111, 5, 0xabcde)),
            Instruction::Lui {
                rd: 5,
                imm: 0xabcde
            }
        );
        // The top of the field must survive.
        assert_eq!(
            Instruction::decode(Enc::u(0b001_0111, 5, 0xfffff)),
            Instruction::Auipc {
                rd: 5,
                imm: 0xfffff
            }
        );
    }

    #[test]
    fn misc_mem_and_unsupported() {
        assert_eq!(Instruction::decode(0x0000_000f), Instruction::Fence);
        assert_eq!(Instruction::decode(0x0000_100f), Instruction::Fence);
        // A zero word is not a valid instruction.
        assert_eq!(Instruction::decode(0x0000_0000), Instruction::Unsupported);
        // Neither is wfi, which would never return.
        assert_eq!(Instruction::decode(0xc00_1073), Instruction::Unsupported);
        // Nor is a random opcode.
        assert_eq!(Instruction::decode(0xffff_ffff), Instruction::Unsupported);
    }

    #[test]
    fn sign_extend_uses_the_field_width_not_the_bit_position() {
        assert_eq!(sign_extend(0xfff, 12), 0xffff_ffff);
        assert_eq!(sign_extend(0x7ff, 12), 0x7ff);
        assert_eq!(sign_extend(0x800, 12), 0xffff_f800);
        assert_eq!(sign_extend(0, 32), 0);
        assert_eq!(sign_extend(0xffff_ffff, 32), 0xffff_ffff);
        assert_eq!(sign_extend(1, 32), 1);
    }
}

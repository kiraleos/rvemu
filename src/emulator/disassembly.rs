// A decoded immediate is a `u32` bit pattern, so the ones that are values
// rather than fields are reinterpreted as the signed numbers they are: an offset
// has to print as `-4` and not as `4294967292`, and a branch target has to be
// computed as a signed addition. That is what the encodings are for, so the
// lints about it are muted here instead of at each of the sites.
#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]

//! The text of a decoded instruction.
//!
//! [`Disassembly`] is what `--debug` prints as a program runs and what
//! `--disassemble` prints for a program that is not running at all. Both go
//! through this one function, so a line of a trace and a line of a listing of
//! the same instruction are the same line, and neither can drift away from what
//! the emulator actually executes.
//!
//! Formatting one is a pure function of the instruction and the address the
//! instruction sits at. The address is the one thing a decoded instruction does
//! not carry: `beq` and `jal` encode an offset, and the address they transfer
//! to is that offset counted from wherever the instruction is.
//!
//! ```
//! use rvemu::emulator::disassembly::Disassembly;
//! use rvemu::emulator::instruction::Instruction;
//!
//! // `jal x0, +0x48` is the first instruction of the program in `tests/add`.
//! // Its offset is 0x48, but at 0x1000 it means 0x1048.
//! let word = 0x0480_006f;
//! let instruction = Instruction::decode(word);
//! assert_eq!(
//!     Disassembly::new(0x1000, &instruction).to_string(),
//!     "jal     x0,00001048",
//! );
//! ```

use super::instruction::{Instruction, SHIFT_AMOUNT_MASK};
use std::fmt;

/// What a word this emulator cannot execute is called, in a trace and in a
/// listing alike. It is the same word the run loop reports as unsupported, so a
/// listing says up front which words a program would stop on.
const UNIMPLEMENTED: &str = "unimp";

/// One decoded instruction, formatted the way a trace and a listing print it.
///
/// The mnemonic occupies the first eight columns and the operands follow it
/// separated by commas, so that a listing lines up down its whole length. An
/// operand that is a *value* — an offset, a shift amount — is printed as a
/// signed decimal number, while one that is a *field* — a CSR number, the twenty
/// bits `lui` shifts into place — is printed in hexadecimal, because that is
/// what it is: a number the instruction copies rather than one it computes with.
///
/// This never panics and never fails, however malformed the encoding is.
/// [`Instruction::decode`] is deliberately permissive, leaving field values it
/// does not recognise to be reported when the instruction is executed, so this
/// has to be able to describe those too: it prints [`UNIMPLEMENTED`] for any
/// field combination the emulator cannot execute.
#[derive(Debug, Clone, Copy)]
pub struct Disassembly<'a> {
    pc: u32,
    instruction: &'a Instruction,
}

impl<'a> Disassembly<'a> {
    /// Formats the instruction as it is disassembled at address `pc`.
    #[must_use]
    pub fn new(pc: u32, instruction: &'a Instruction) -> Self {
        Disassembly { pc, instruction }
    }

    /// The address a control transfer instruction goes to, given the offset it
    /// encodes. The offset is a signed value added to the program counter of the
    /// instruction itself, not of the one that follows it.
    fn target(&self, offset: u32) -> u32 {
        self.pc.wrapping_add(offset)
    }
}

impl fmt::Display for Disassembly<'_> {
    // One arm per format, and every one of them is a different set of fields to
    // name. There is nothing to shorten here that would not make it harder to
    // read against the manual.
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self.instruction {
            Instruction::R {
                rd,
                funct3,
                rs1,
                rs2,
                funct7,
            } => {
                let mnemonic = match (funct3, funct7) {
                    // `add` and `sub` share funct3 and are told apart by funct7,
                    // and so are `srl` and `sra`. Every other funct3 has a
                    // mnemonic of its own whatever funct7 says, because the top
                    // seven bits of those encodings have to be zero and this
                    // function does not care.
                    (0b000, 0b000_0000) => "add",
                    (0b000, 0b010_0000) => "sub",
                    (0b001, _) => "sll",
                    (0b010, _) => "slt",
                    (0b011, _) => "sltu",
                    (0b100, _) => "xor",
                    (0b101, 0b000_0000) => "srl",
                    (0b101, 0b010_0000) => "sra",
                    (0b110, _) => "or",
                    (0b111, _) => "and",
                    _ => return f.write_str(UNIMPLEMENTED),
                };
                write!(f, "{mnemonic:<8}x{rd},x{rs1},x{rs2}")
            }
            Instruction::OpImm {
                rd,
                funct3,
                rs1,
                imm,
            } => {
                // `addi x0, x0, 0` is the canonical encoding of a nop.
                if rd == 0 && rs1 == 0 && imm == 0 {
                    return f.write_str("nop");
                }
                let mnemonic = match funct3 {
                    0b000 => "addi",
                    0b001 => "slli",
                    0b010 => "slti",
                    0b011 => "sltiu",
                    0b100 => "xori",
                    0b101 => match imm >> 5 & 0b111_1111 {
                        0b000_0000 => "srli",
                        0b010_0000 => "srai",
                        _ => return f.write_str(UNIMPLEMENTED),
                    },
                    0b110 => "ori",
                    0b111 => "andi",
                    _ => return f.write_str(UNIMPLEMENTED),
                };
                // A shift amount is the low five bits of the immediate, and is
                // displayed as the number of places to shift by. `sltiu` compares
                // its register against the sign extended immediate read as an
                // unsigned value, and is displayed that way. Every other form of
                // this format applies a signed value.
                match funct3 {
                    0b001 | 0b101 => {
                        write!(f, "{mnemonic:<8}x{rd},x{rs1},{}", imm & SHIFT_AMOUNT_MASK)
                    }
                    0b011 => write!(f, "{mnemonic:<8}x{rd},x{rs1},{imm}"),
                    _ => write!(f, "{mnemonic:<8}x{rd},x{rs1},{}", imm as i32),
                }
            }
            Instruction::Load {
                rd,
                funct3,
                rs1,
                imm,
            } => {
                let mnemonic = match funct3 {
                    0b000 => "lb",
                    0b001 => "lh",
                    0b010 => "lw",
                    0b100 => "lbu",
                    0b101 => "lhu",
                    _ => return f.write_str(UNIMPLEMENTED),
                };
                // The offset is signed whichever way the loaded bytes are then
                // interpreted: `lbu` differs from `lb` in not sign extending the
                // value, not in how it is addressed.
                write!(f, "{mnemonic:<8}x{rd},{}(x{rs1})", imm as i32)
            }
            Instruction::Store {
                imm,
                funct3,
                rs1,
                rs2,
            } => {
                let mnemonic = match funct3 {
                    0b000 => "sb",
                    0b001 => "sh",
                    0b010 => "sw",
                    _ => return f.write_str(UNIMPLEMENTED),
                };
                write!(f, "{mnemonic:<8}x{rs2},{}(x{rs1})", imm as i32)
            }
            Instruction::Branch {
                rs1,
                rs2,
                imm,
                funct3,
            } => {
                let mnemonic = match funct3 {
                    0b000 => "beq",
                    0b001 => "bne",
                    0b100 => "blt",
                    0b101 => "bge",
                    0b110 => "bltu",
                    0b111 => "bgeu",
                    _ => return f.write_str(UNIMPLEMENTED),
                };
                // A branch is printed as the address it goes to rather than as
                // the offset it encodes, so that the targets in a listing can be
                // read off the listing itself.
                let target = self.target(imm);
                write!(f, "{mnemonic:<8}x{rs1},x{rs2},{target:08x}")
            }
            Instruction::Jump { rd, imm } => {
                // As for a branch, the address rather than the offset.
                let target = self.target(imm);
                write!(f, "jal     x{rd},{target:08x}")
            }
            Instruction::JumpRegister { rd, rs1, imm } => {
                // The exception to the rule above: a `jalr` target depends on a
                // register, so it is not known until the instruction runs. Its
                // offset is what can be printed, and it is an offset.
                write!(f, "jalr    x{rd},x{rs1},{}", imm as i32)
            }
            Instruction::Lui { rd, imm } => {
                // The twenty bits that go above the low twelve, which is what
                // this instruction's immediate is: a field, not a value.
                write!(f, "lui     x{rd},{imm:#x}")
            }
            Instruction::Auipc { rd, imm } => {
                write!(f, "auipc   x{rd},{imm:#x}")
            }
            Instruction::System {
                rd,
                funct3,
                rs1,
                imm,
            } => {
                // `ecall`, `ebreak` and `mret` share funct3 and are told apart by
                // the immediate, which for them is a plain twelve bit field
                // rather than a sign extended one. None of them takes operands.
                if funct3 == 0b000 {
                    return match imm {
                        0x000 => f.write_str("ecall"),
                        0x001 => f.write_str("ebreak"),
                        0x302 => f.write_str("mret"),
                        _ => f.write_str(UNIMPLEMENTED),
                    };
                }
                let mnemonic = match funct3 {
                    0b001 => "csrrw",
                    0b010 => "csrrs",
                    0b011 => "csrrc",
                    0b101 => "csrrwi",
                    0b110 => "csrrsi",
                    0b111 => "csrrci",
                    _ => return f.write_str(UNIMPLEMENTED),
                };
                // A CSR number is a twelve bit field the decoder sign extended
                // along with the rest of the immediate, so the extension comes
                // back off before it is printed.
                let csr = imm & 0xfff;
                // The last three forms take a zero extended five bit immediate
                // where the others take a register, so it is displayed as the
                // number it is rather than as one of `x0` to `x31`.
                if funct3 >= 0b101 {
                    write!(f, "{mnemonic:<8}x{rd},{csr:#x},{rs1}")
                } else {
                    write!(f, "{mnemonic:<8}x{rd},{csr:#x},x{rs1}")
                }
            }
            Instruction::Fence { funct3 } => {
                // The two defined forms of this opcode, and the reserved
                // encodings, which are no-ops here along with both of them.
                f.write_str(if funct3 == 0b001 { "fence.i" } else { "fence" })
            }
            Instruction::Unsupported => f.write_str(UNIMPLEMENTED),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::instruction::enc;

    /// The disassembly of one encoded word, placed at address zero so that a
    /// branch or a jump target is the offset itself.
    fn disasm(word: u32) -> String {
        Disassembly::new(0, &Instruction::decode(word)).to_string()
    }

    #[test]
    fn every_format_names_its_operands() {
        // The two halves of each pair, so that both are covered.
        for (funct7, mnemonic) in [(0b000_0000, "add"), (0b010_0000, "sub")] {
            assert_eq!(
                disasm(enc::r(funct7, 3, 2, 0b000, 1)),
                format!("{mnemonic:<8}x1,x2,x3")
            );
        }
        for (funct3, mnemonic) in [
            (0b001, "sll"),
            (0b010, "slt"),
            (0b011, "sltu"),
            (0b100, "xor"),
            (0b110, "or"),
            (0b111, "and"),
        ] {
            assert_eq!(
                disasm(enc::r(0, 3, 2, funct3, 1)),
                format!("{mnemonic:<8}x1,x2,x3")
            );
        }
        for (funct7, mnemonic) in [(0b000_0000, "srl"), (0b010_0000, "sra")] {
            assert_eq!(
                disasm(enc::r(funct7, 3, 2, 0b101, 1)),
                format!("{mnemonic:<8}x1,x2,x3")
            );
        }
        for (funct3, mnemonic) in [
            (0b000, "addi"),
            (0b010, "slti"),
            (0b100, "xori"),
            (0b110, "ori"),
            (0b111, "andi"),
        ] {
            assert_eq!(
                disasm(enc::op_imm(1, funct3, 2, -4)),
                format!("{mnemonic:<8}x1,x2,-4")
            );
        }
        assert_eq!(disasm(enc::op_imm(1, 0b001, 2, 4)), "slli    x1,x2,4");
        assert_eq!(disasm(enc::op_imm(1, 0b100, 2, -4)), "xori    x1,x2,-4");
        // The two right shifts differ only in the top seven bits of the
        // immediate, and print the same shift amount either way.
        assert_eq!(disasm(enc::op_imm(1, 0b101, 2, 4)), "srli    x1,x2,4");
        assert_eq!(
            disasm(enc::op_imm(1, 0b101, 2, 0b0100_0000_0100)),
            "srai    x1,x2,4",
        );
        for (funct3, mnemonic) in [
            (0b000, "lb"),
            (0b001, "lh"),
            (0b010, "lw"),
            (0b100, "lbu"),
            (0b101, "lhu"),
        ] {
            assert_eq!(
                disasm(enc::load(1, funct3, 2, -4)),
                format!("{mnemonic:<8}x1,-4(x2)")
            );
        }
        for (funct3, mnemonic) in [(0b000, "sb"), (0b001, "sh"), (0b010, "sw")] {
            assert_eq!(
                disasm(enc::store(funct3, 2, 3, -4)),
                format!("{mnemonic:<8}x3,-4(x2)")
            );
        }
        for (funct3, mnemonic) in [
            (0b000, "beq"),
            (0b001, "bne"),
            (0b100, "blt"),
            (0b101, "bge"),
            (0b110, "bltu"),
            (0b111, "bgeu"),
        ] {
            assert_eq!(
                disasm(enc::branch(funct3, 2, 3, 8)),
                format!("{mnemonic:<8}x2,x3,00000008"),
            );
        }
        assert_eq!(disasm(enc::jal(1, 8)), "jal     x1,00000008");
        assert_eq!(disasm(enc::jalr(1, 2, -4)), "jalr    x1,x2,-4");
        assert_eq!(disasm(enc::lui(1, 0xabcde)), "lui     x1,0xabcde");
        assert_eq!(disasm(enc::auipc(1, 0xabcde)), "auipc   x1,0xabcde");
        assert_eq!(disasm(enc::op_imm(0, 0b000, 0, 0)), "nop");
    }

    #[test]
    fn a_branch_and_a_jump_print_the_address_they_go_to() {
        // The offset is relative to the instruction, so the same word at two
        // addresses names two targets, and a negative one wraps below zero.
        let word = enc::branch(0b000, 2, 3, -8);
        assert_eq!(
            Disassembly::new(0x1000, &Instruction::decode(word)).to_string(),
            "beq     x2,x3,00000ff8",
        );
        let word = enc::jal(1, -8);
        assert_eq!(
            Disassembly::new(0x1000, &Instruction::decode(word)).to_string(),
            "jal     x1,00000ff8",
        );
    }

    #[test]
    fn an_offset_is_a_signed_value_whatever_the_instruction_does_with_it() {
        // `lbu` and `lhu` are `lb` and `lh` with the loaded value left
        // unsigned. Their offset is the same signed field either way, so a
        // negative one must not print as the huge unsigned number it also is.
        assert_eq!(disasm(enc::load(1, 0b100, 2, -1)), "lbu     x1,-1(x2)");
        assert_eq!(disasm(enc::load(1, 0b101, 2, -1)), "lhu     x1,-1(x2)");
        assert_eq!(disasm(enc::store(0b000, 2, 3, -1)), "sb      x3,-1(x2)");
        // `sltiu` is the exception: it compares against the sign extended
        // immediate read as an unsigned value, so it is displayed that way.
        assert_eq!(
            disasm(enc::op_imm(1, 0b011, 2, -1)),
            "sltiu   x1,x2,4294967295"
        );
    }

    #[test]
    fn a_csr_number_is_a_field_and_an_immediate_form_is_not_a_register() {
        assert_eq!(
            disasm(enc::system(1, 0b001, 2, 0x300)),
            "csrrw   x1,0x300,x2"
        );
        assert_eq!(
            disasm(enc::system(1, 0b010, 2, 0xc00)),
            "csrrs   x1,0xc00,x2"
        );
        // The decoder sign extends the twelve bit field, so the top bit of a
        // CSR number has to come back off before it is printed.
        assert_eq!(
            disasm(enc::system(1, 0b001, 2, 0xfff)),
            "csrrw   x1,0xfff,x2"
        );
        // The last three forms carry a zero extended five bit number in the rs1
        // field, which is an immediate rather than a register.
        assert_eq!(
            disasm(enc::system(1, 0b101, 5, 0x300)),
            "csrrwi  x1,0x300,5"
        );
        assert_eq!(
            disasm(enc::system(1, 0b110, 31, 0x300)),
            "csrrsi  x1,0x300,31"
        );
        assert_eq!(
            disasm(enc::system(1, 0b111, 0, 0x300)),
            "csrrci  x1,0x300,0"
        );
    }

    #[test]
    fn the_traps_and_the_fences_are_named() {
        assert_eq!(disasm(enc::system(0, 0b000, 0, 0x000)), "ecall");
        assert_eq!(disasm(enc::system(0, 0b000, 0, 0x001)), "ebreak");
        assert_eq!(disasm(enc::system(0, 0b000, 0, 0x302)), "mret");
        // The funct3 is what tells the two fences apart, and it is the only
        // thing that does.
        assert_eq!(disasm(enc::fence(0b000)), "fence");
        assert_eq!(disasm(enc::fence(0b001)), "fence.i");
    }

    #[test]
    fn a_field_this_emulator_cannot_execute_is_unimp() {
        // Decoding is deliberately permissive, so every one of these decodes to a
        // real instruction with a field value that is not one. The run loop
        // refuses to execute them, and a listing has to say so rather than
        // pretend otherwise, or panic on a whole segment of memory. Note that
        // every funct3 of `OP` and `OP-IMM` is a mnemonic and a funct3 is three
        // bits, so only funct7 is unknown in those two formats.
        for word in [
            0x0000_0000,                                // not an encoding at all
            0xc00_1073,                                 // wfi, which would never return
            enc::r(0b000_0001, 2, 1, 0b000, 1),         // an unknown funct7
            enc::op_imm(1, 0b101, 1, 0b0000_0100_0100), // an unknown shift funct7
            enc::load(1, 0b011, 1, 0),                  // an unknown funct3
            enc::store(0b011, 1, 2, 0),                 // an unknown funct3
            enc::branch(0b010, 1, 2, 8),                // an unknown funct3
            enc::system(0, 0b100, 0, 0),                // a reserved funct3
            enc::system(0, 0b000, 0, 0x123),            // not ecall, ebreak or mret
        ] {
            assert_eq!(disasm(word), UNIMPLEMENTED, "{word:#010x}");
        }
    }
}

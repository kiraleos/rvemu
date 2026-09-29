/// Why a single instruction could not be executed.
///
/// Traps model conditions caused by the *guest* program, so they are expected
/// input rather than emulator bugs: a well-formed emulator reports them and
/// stops, instead of panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trap {
    /// The encoding is not a valid RV32I instruction, or belongs to an
    /// extension this emulator does not implement.
    UnsupportedInstruction { detail: String },
    /// `ecall` requested a system call the emulator does not implement.
    UnsupportedSyscall { num: u32 },
    /// The instruction is a known-but-unimplemented placeholder (`0x00000000`
    /// and `0xc0001073`), used by test binaries to signal a trap.
    Unimplemented,
}

impl Trap {
    /// The process exit code reported when the program stops on this trap.
    pub fn exit_code(&self) -> i32 {
        match self {
            Trap::UnsupportedInstruction { .. } => -2,
            Trap::UnsupportedSyscall { .. } => -2,
            Trap::Unimplemented => -3,
        }
    }

    /// The human-readable line printed when `--debug` is set.
    pub fn message(&self) -> String {
        match self {
            Trap::UnsupportedInstruction { detail } => {
                format!("Unsupported instruction: {}", detail)
            }
            Trap::UnsupportedSyscall { num } => {
                format!("Unimplemented ECALL: {}", num)
            }
            Trap::Unimplemented => "Reached an unimp instruction.".to_string(),
        }
    }
}

/// Why a binary could not be loaded into the emulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The file could not be opened or read.
    Io(String),
    /// The file is not a usable ELF image.
    Elf(String),
    /// The ELF targets a machine this emulator does not implement.
    Arch(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(msg) => write!(f, "{}", msg),
            LoadError::Elf(msg) => write!(f, "{}", msg),
            LoadError::Arch(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

/// The result of executing one instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Keep fetching from the next `pc`.
    Continue,
    /// The guest invoked the `exit` system call with the given status.
    Exit(i32),
}

#[allow(unused_imports)]
use crate::Args;
#[allow(unused_imports)]
use crate::Cpu;
#[test]
fn add() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/add").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn addi() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/addi").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn and() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/and").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn andi() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/andi").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn auipc() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/auipc").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn beq() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/beq").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bge() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/bge").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bgeu() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/bgeu").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn blt() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/blt").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bltu() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/bltu").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bne() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/bne").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn fence_i() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/fence_i").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn jal() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/jal").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn jalr() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/jalr").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lb() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/lb").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lbu() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/lbu").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lh() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/lh").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lhu() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/lhu").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lui() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/lui").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lw() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/lw").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn or() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/or").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn ori() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/ori").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sb() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sb").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sh() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sh").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn simple() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/simple").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sll() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sll").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn slli() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/slli").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn slt() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/slt").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn slti() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/slti").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sltiu() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sltiu").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sltu() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sltu").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sra() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sra").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn srai() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/srai").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn srl() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/srl").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn srli() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/srli").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sub() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sub").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sw() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/sw").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn xor() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/xor").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn xori() {
    let mut cpu = Cpu::new(16).unwrap();
    let args = Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: None,
    };
    cpu.load("./tests/xori").unwrap();
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn parse_hex_accepts_bare_and_prefixed() {
    assert_eq!(crate::parse_hex("0").unwrap(), 0);
    assert_eq!(crate::parse_hex("1000").unwrap(), 0x1000);
    assert_eq!(crate::parse_hex("0x1000").unwrap(), 0x1000);
    assert_eq!(crate::parse_hex("0X1000").unwrap(), 0x1000);
    assert_eq!(crate::parse_hex("ffffffff").unwrap(), 0xffff_ffff);
}

#[test]
fn parse_hex_rejects_junk_and_overflow() {
    assert!(crate::parse_hex("zzzz").is_err());
    assert!(crate::parse_hex("").is_err());
    assert!(crate::parse_hex("0x").is_err());
    // 33 bits does not fit a u32.
    assert!(crate::parse_hex("1ffffffff").is_err());
}

#[test]
fn mem_kib_defaults_and_rejects_nonsense() {
    let args = |mem: Option<&str>| Args {
        file: std::path::PathBuf::new(),
        debug: false,
        registers: false,
        aliases: false,
        interactive: false,
        pc: None,
        stack: false,
        mem: mem.map(String::from),
    };

    assert_eq!(crate::mem_kib(&args(None)).unwrap(), 16);
    assert_eq!(crate::mem_kib(&args(Some("64"))).unwrap(), 64);
    assert!(crate::mem_kib(&args(Some("banana"))).is_err());
    assert!(crate::mem_kib(&args(Some("0"))).is_err());
    assert!(crate::mem_kib(&args(Some("-1"))).is_err());
    // Large enough to overflow the KiB-to-byte multiply.
    assert!(crate::mem_kib(&args(Some("99999999999999999999"))).is_err());
}

#[test]
fn cpu_new_reports_unallocatable_memory() {
    assert!(Cpu::new(0).is_ok());
    assert!(Cpu::new(16).is_ok());
    // ~97 GiB: valid arithmetic, but not allocatable.
    assert!(Cpu::new(100_000_000).is_err());
}

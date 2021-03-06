#[allow(unused_imports)]
use crate::Args;
#[allow(unused_imports)]
use crate::Cpu;
#[test]
fn add() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/add");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn addi() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/addi");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn and() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/and");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn andi() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/andi");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn auipc() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/auipc");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn beq() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/beq");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bge() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/bge");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bgeu() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/bgeu");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn blt() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/blt");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bltu() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/bltu");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn bne() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/bne");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn fence_i() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/fence_i");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn jal() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/jal");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn jalr() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/jalr");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lb() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/lb");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lbu() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/lbu");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lh() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/lh");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lhu() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/lhu");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lui() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/lui");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn lw() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/lw");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn or() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/or");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn ori() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/ori");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sb() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sb");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sh() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sh");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn simple() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/simple");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sll() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sll");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn slli() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/slli");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn slt() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/slt");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn slti() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/slti");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sltiu() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sltiu");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sltu() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sltu");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sra() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sra");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn srai() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/srai");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn srl() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/srl");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn srli() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/srli");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sub() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sub");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn sw() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/sw");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn xor() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/xor");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

#[test]
fn xori() {
    let mut cpu = Cpu::new(16);
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
    cpu.load("./tests/xori");
    let ret = cpu.run(args);
    assert_eq!(ret, 0);
}

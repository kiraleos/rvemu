use crate::emulator::cpu::{Cpu, Outcome, RunConfig};
#[test]
fn add() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/add");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn addi() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/addi");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn and() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/and");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn andi() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/andi");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn auipc() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/auipc");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn beq() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/beq");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bge() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bge");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bgeu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bgeu");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn blt() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/blt");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bltu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bltu");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bne() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bne");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn fence_i() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/fence_i");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn jal() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/jal");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn jalr() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/jalr");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lb() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lb");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lbu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lbu");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lh() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lh");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lhu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lhu");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lui() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lui");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lw() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lw");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn or() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/or");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn ori() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/ori");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sb() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sb");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sh() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sh");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn simple() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/simple");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sll() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sll");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn slli() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/slli");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn slt() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/slt");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn slti() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/slti");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sltiu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sltiu");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sltu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sltu");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sra() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sra");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn srai() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/srai");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn srl() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/srl");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn srli() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/srli");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sub() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sub");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sw() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sw");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn xor() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/xor");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn xori() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/xori");
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

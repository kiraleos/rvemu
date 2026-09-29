use crate::emulator::cpu::{Cpu, Outcome, RunConfig};
#[test]
fn add() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/add").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn addi() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/addi").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn and() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/and").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn andi() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/andi").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn auipc() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/auipc").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn beq() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/beq").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bge() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bge").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bgeu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bgeu").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn blt() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/blt").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bltu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bltu").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn bne() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/bne").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn fence_i() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/fence_i").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn jal() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/jal").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn jalr() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/jalr").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lb() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lb").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lbu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lbu").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lh() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lh").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lhu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lhu").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lui() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lui").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn lw() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/lw").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn or() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/or").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn ori() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/ori").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sb() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sb").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sh() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sh").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn simple() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/simple").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sll() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sll").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn slli() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/slli").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn slt() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/slt").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn slti() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/slti").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sltiu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sltiu").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sltu() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sltu").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sra() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sra").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn srai() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/srai").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn srl() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/srl").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn srli() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/srli").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sub() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sub").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn sw() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/sw").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn xor() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/xor").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

#[test]
fn xori() {
    let mut cpu = Cpu::new(16);
    let config = RunConfig::default();
    cpu.load("./tests/xori").unwrap();
    assert_eq!(cpu.run(&config), Outcome::Exited(0));
}

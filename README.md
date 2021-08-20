# RISC-V (RV32I) Emulator
A RISC-V emulator, specifically the RV32I base integer instruction set.
Made entirely to learn how to write an emulator and gain experience with Rust.

At this moment, this emulator prints the instructions being executed, and when it reaches an `unimp` instruction or the program counter overflows the emulation stops and the cpu's registers are printed. Syscalls (`ecall` instructions) are also not yet implemented.

Tests are also included in this repo. The tests are built from [riscv-tests](https://github.com/riscv/riscv-tests). Currently, even the rv32ui-p-* tests include instructions that are not part of the RV32I instruction set for some reason, so there are some unimplemented instructions, like `fence` and the `CSR` instruction set.
## Build
You need `rust` and `cargo` installed in order to build the emulator.
```bash
$ git clone https://github.com/kiraleos/riscv32i-emulator.git
$ cargo build --release
```
## Usage
```bash
$ ./target/release/riscv-emulator <file>
```
or
```bash
$ cargo run --release <file>
```

### Example usage
```bash
$ cargo run --release ./tests/add
```

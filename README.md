# RISC-V (RV32I) Emulator
A RISC-V emulator, specifically the RV32I base integer instruction set.
Made entirely to learn how to write an emulator and gain experience with Rust.

At this moment, this emulator prints the instructions being executed, and when it reaches an `unimp` instruction or the program counter overflows the emulation stops and the cpu's registers are printed. Syscalls (`ecall` instructions) are also not yet implemented.
## Build
You need `rust` and `cargo` installed in order to build the emulator.
```
$ git clone https://github.com/kiraleos/riscv32i-emulator.git
$ cargo build --release
```
## Usage
```
$ ./target/release/riscv-emulator <file>
```
or
```
$ cargo run --release <file>
```

### Example usage
```
$ cargo run --release ./tests/add
```

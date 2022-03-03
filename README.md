# RISC-V (RV32I) Emulator
A RISC-V emulator, specifically the RV32I base integer instruction set.
Made entirely to learn how to write an emulator and gain experience with Rust.

At this moment, this emulator has only been tested on the provided test binaries, but any file compiled for 32bit little-endian RISC-V should work. Also, only the `exit` syscall is implemented.

The pre-compiled test binaries are included in this repo. The tests are built from [riscv-tests](https://github.com/riscv/riscv-tests).
## Build & Run
You need `rust` and `cargo` installed in order to build the emulator.
```bash
$ git clone https://github.com/kiraleos/riscv32i-emulator.git
$ cd riscv32i-emulator
$ cargo run ./tests/<file>
```
or
```bash
$ cargo test
```
to run the unit tests.

## Flags
Use the `-p` flag to print each executed instruction and the `-d` flag to print some debug info.
### Example
```bash
$ cargo run -q ./tests/simple -p
00000000:   0480006f      jal     x0,00000048
00000048:   00000093      addi    x1,x0,0
0000004c:   00000113      addi    x2,x0,0
00000050:   00000193      addi    x3,x0,0
00000054:   00000213      addi    x4,x0,0
00000058:   00000293      addi    x5,x0,0
0000005c:   00000313      addi    x6,x0,0
00000060:   00000393      addi    x7,x0,0
00000064:   00000413      addi    x8,x0,0
00000068:   00000493      addi    x9,x0,0
0000006c:   00000513      addi    x10,x0,0
00000070:   00000593      addi    x11,x0,0
00000074:   00000613      addi    x12,x0,0
00000078:   00000693      addi    x13,x0,0
0000007c:   00000713      addi    x14,x0,0
00000080:   00000793      addi    x15,x0,0
00000084:   00000813      addi    x16,x0,0
00000088:   00000893      addi    x17,x0,0
0000008c:   00000913      addi    x18,x0,0
00000090:   00000993      addi    x19,x0,0
00000094:   00000a13      addi    x20,x0,0
00000098:   00000a93      addi    x21,x0,0
0000009c:   00000b13      addi    x22,x0,0
000000a0:   00000b93      addi    x23,x0,0
000000a4:   00000c13      addi    x24,x0,0
000000a8:   00000c93      addi    x25,x0,0
000000ac:   00000d13      addi    x26,x0,0
000000b0:   00000d93      addi    x27,x0,0
000000b4:   00000e13      addi    x28,x0,0
000000b8:   00000e93      addi    x29,x0,0
000000bc:   00000f13      addi    x30,x0,0
000000c0:   00000f93      addi    x31,x0,0
000000c4:   f1402573      csrrs   x10,0xffffff14,x0
000000c8:   00051063      bne     x10,x0,000010c8
000000cc:   00000297      auipc   x5,0x0
000000d0:   01028293      addi    x5,x5,16
000000d4:   30529073      csrrw   x0,0x305,x5
000000d8:   18005073      csrrwi  x0,0x180,0
000000dc:   00000297      auipc   x5,0x0
000000e0:   02028293      addi    x5,x5,32
000000e4:   30529073      csrrw   x0,0x305,x5
000000e8:   800002b7      lui     x5,0x80000
000000ec:   fff28293      addi    x5,x5,-1
000000f0:   3b029073      csrrw   x0,0x3b0,x5
000000f4:   01f00293      addi    x5,x0,31
000000f8:   3a029073      csrrw   x0,0x3a0,x5
000000fc:   30405073      csrrwi  x0,0x304,0
00000100:   00000297      auipc   x5,0x0
00000104:   01428293      addi    x5,x5,20
00000108:   30529073      csrrw   x0,0x305,x5
0000010c:   30205073      csrrwi  x0,0x302,0
00000110:   30305073      csrrwi  x0,0x303,0
00000114:   00000193      addi    x3,x0,0
00000118:   00000297      auipc   x5,0x0
0000011c:   eec28293      addi    x5,x5,-276
00000120:   30529073      csrrw   x0,0x305,x5
00000124:   00100513      addi    x10,x0,1
00000128:   01f51513      slli    x10,x10,0x1f
0000012c:   00054c63      blt     x10,x0,00001144
00000144:   00000293      addi    x5,x0,0
00000148:   00028a63      beq     x5,x0,0000115c
0000015c:   30005073      csrrwi  x0,0x300,0
00000160:   00000297      auipc   x5,0x0
00000164:   01428293      addi    x5,x5,20
00000168:   34129073      csrrw   x0,0x341,x5
0000016c:   f1402573      csrrs   x10,0xffffff14,x0
00000170:   30200073      unknown ecall/ebreak imm: 0b001100000010
00000174:   0ff0000f      fence
00000178:   00100193      addi    x3,x0,1
0000017c:   05d00893      addi    x17,x0,93
00000180:   00000513      addi    x10,x0,0
00000184:   00000073      ecall
./tests/simple
        exit code: 0
```
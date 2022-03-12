# RVemu, a RISC-V emulator
A RISC-V emulator, specifically the RV32I base integer instruction set.

This emulator does not provide any kernel or OS, so programs that expect a kernel or an OS will not work as expected. The only thing close to a kernel that this emulator provides is the `exit()` system call and the `--stack` option which provides a stack space. With these two features, this emulator can effectively execute compiled binaries that do not rely on `libc`. 

The pre-compiled test binaries are included in this repo. The tests are built from [riscv-tests](https://github.com/riscv/riscv-tests). All the tests pass, so every RV32I instruction works as per the specification.

```
$ cargo test -q

running 39 tests
.......................................
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Build & Run
You need `rust` and `cargo` installed in order to build the emulator.
```
$ git clone https://github.com/kiraleos/rvemu.git
$ cd rvemu
$ cargo run ./tests/<file>
```
or, to run the unit tests
```
$ cargo test
```
## Usage
```
USAGE:
    rvemu [OPTIONS] <FILE>

ARGS:
    <FILE>    The path of the file to be executed

OPTIONS:
    -a, --aliases         Show register ABI names or numeric values (x0-x31) Use with the
                          `--registers` option
    -d, --debug           Print instructions as they are executed
    -h, --help            Print help information
    -i, --interactive     Interactive mode. Use with either `--registers` and/or `--debug`
        --mem <size>      Set memory size in KiB (default = 16KiB)
        --pc <address>    Override ELF entry point
    -r, --registers       Show register values after each instruction
    -s, --stack           Provide a stack of "infinite" size. This sets the stack pointer before
                          execution, so it might cause undefined behaviour
    -V, --version         Print version information
```
## Interactive mode
To launch the emulator in interactive mode, pass the `-i` or `--interactive` option.

This mode currently supports 2 commands: 
* To see the contents of a register:
    
    `reg 5`
* To see the contents of a memory location (physical address in hex):

    `mem 0123abcd`

To execute the next instruction just press enter. 
## Example
```
$ cargo run -q -- ./tests/simple --debug --interactive --registers --aliases
> 
  pc: 0x00001048
zero: 0x00000000    ra: 0x00000000    sp: 0x00000000    gp: 0x00000000  
  tp: 0x00000000    t0: 0x00000000    t1: 0x00000000    t2: 0x00000000  
  s0: 0x00000000    s1: 0x00000000    a0: 0x00000000    a1: 0x00000000  
  a2: 0x00000000    a3: 0x00000000    a4: 0x00000000    a5: 0x00000000  
  a6: 0x00000000    a7: 0x00000000    s2: 0x00000000    s3: 0x00000000  
  s4: 0x00000000    s5: 0x00000000    s6: 0x00000000    s7: 0x00000000  
  s8: 0x00000000    s9: 0x00000000   s10: 0x00000000   s11: 0x00000000  
  t3: 0x00000000    t4: 0x00000000    t5: 0x00000000    t6: 0x00000000  

00001000:   0480006f            jal     x0,00000048
> 
  pc: 0x0000104c
zero: 0x00000000    ra: 0x00000000    sp: 0x00000000    gp: 0x00000000  
  tp: 0x00000000    t0: 0x00000000    t1: 0x00000000    t2: 0x00000000  
  s0: 0x00000000    s1: 0x00000000    a0: 0x00000000    a1: 0x00000000  
  a2: 0x00000000    a3: 0x00000000    a4: 0x00000000    a5: 0x00000000  
  a6: 0x00000000    a7: 0x00000000    s2: 0x00000000    s3: 0x00000000  
  s4: 0x00000000    s5: 0x00000000    s6: 0x00000000    s7: 0x00000000  
  s8: 0x00000000    s9: 0x00000000   s10: 0x00000000   s11: 0x00000000  
  t3: 0x00000000    t4: 0x00000000    t5: 0x00000000    t6: 0x00000000  

00001048:   00000093            addi    x1,x0,0
> reg 2
0x0
> mem 104c
0x00000113
> 
  pc: 0x00001050
zero: 0x00000000    ra: 0x00000000    sp: 0x00000000    gp: 0x00000000  
  tp: 0x00000000    t0: 0x00000000    t1: 0x00000000    t2: 0x00000000  
  s0: 0x00000000    s1: 0x00000000    a0: 0x00000000    a1: 0x00000000  
  a2: 0x00000000    a3: 0x00000000    a4: 0x00000000    a5: 0x00000000  
  a6: 0x00000000    a7: 0x00000000    s2: 0x00000000    s3: 0x00000000  
  s4: 0x00000000    s5: 0x00000000    s6: 0x00000000    s7: 0x00000000  
  s8: 0x00000000    s9: 0x00000000   s10: 0x00000000   s11: 0x00000000  
  t3: 0x00000000    t4: 0x00000000    t5: 0x00000000    t6: 0x00000000  

0000104c:   00000113            addi    x2,x0,0
> 
```
## Cross-compiling C for RISC-V
You might want to compile your own C code for RISC-V instead of just running the provided tests.

To do that you need to: 
1. Install the [riscv-gnu-toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain) 
2. Write your C program (without stdlib). For example:
    ```c
    int fib(int n) {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    }

    void _start() {
        fib(30);

        // exit syscall
        asm("addi a7,zero,93;"
            "addi a0,zero,0;"
            "ecall;");
    }
    ```
3. Compile it with `riscv32-unknown-elf-gcc fib.c -o fib -nostdlib`
4. Run the emulator with `fib` as input
    ```
    $ time ./rvemu fib --stack
    Program exited with exit code: 0

    real    0m7.811s
    user    0m7.811s
    sys     0m0.000s
    ```
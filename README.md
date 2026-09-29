# RVemu, a RISC-V emulator
A RISC-V emulator, specifically the RV32I base integer instruction set.

This emulator does not provide any kernel or OS, so programs that expect a kernel or an OS will not work as expected. The only thing close to a kernel that this emulator provides is the `exit()` system call and the `--stack` option which provides a stack space. With these two features, this emulator can effectively execute compiled binaries that do not rely on `libc`. 

The pre-compiled test binaries are included in this repo. The tests are built from [riscv-tests](https://github.com/riscv/riscv-tests). All the tests pass, so every RV32I instruction works as per the specification.

```
$ cargo test

running 47 tests
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

The 47 unit tests cover instruction decoding, execution and disassembly, the 3
integration tests run and list every program in `tests/`, and the two doc tests
are the examples in this file and in the source.

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
Usage: rvemu [OPTIONS] <FILE>

Arguments:
  <FILE>  The path of the file to be executed

Options:
  -d, --debug         Print instructions as they are executed
  -D, --disassemble   Disassemble the program instead of running it
  -r, --registers     Show register values after each instruction
  -a, --aliases       Show register ABI names or numeric values (x0-x31) Use with the `--registers` option
  -i, --interactive   Interactive mode. Use with either `--registers` and/or `--debug`
      --pc <address>  Override ELF entry point, in hexadecimal
  -s, --stack         Provide a stack of "infinite" size. This sets the stack pointer before execution, so it might cause undefined behaviour
      --mem <size>    Set memory size in KiB [default: 16]
  -h, --help          Print help
  -V, --version       Print version
```

The emulator exits with the status the program passed to `exit`, or with 1 if it
could not run: if the program ran off the end of memory, hit a breakpoint, made a
system call that is not implemented, or reached an instruction this emulator does
not implement.

`ebreak` is a requested trap rather than a no-op, so a program that reaches one
stops with a message naming its address. Note that the programs in `tests/` use
`mret` and the machine-mode CSRs as part of their setup, and this emulator has
neither privilege modes nor a CSR file, so it lets `mret` retire and discards
every CSR access. That is what lets them run at all; it also means those programs
only exercise the arithmetic and memory instructions, not traps.

## Disassembly
Pass the `-D` or `--disassemble` option to list a program's code instead of
running it. Every four-byte word of the segment holding the entry point gets a
line: the address it is at, the word it encoded as, and what it means.

```
$ cargo run -q -- -D tests/simple
00001000:   0480006f          	jal     x0,00001048
00001004:   34202f73          	csrrs   x30,0x342,x0
00001008:   00800f93          	addi    x31,x0,8
0000100c:   03ff0863          	beq     x30,x31,0000103c
00001010:   00900f93          	addi    x31,x0,9
00001014:   03ff0463          	beq     x30,x31,0000103c
00001018:   00b00f93          	addi    x31,x0,11
0000101c:   03ff0063          	beq     x30,x31,0000103c
00001020:   00000f13          	addi    x30,x0,0
00001024:   000f0463          	beq     x30,x0,0000102c
00001028:   000f0067          	jalr    x0,x30,0
```

A line is the address, the word, a tab, and the instruction. The three parts come
apart the way the manual describes them. In `0x0480006f` the last seven bits are
`1101111`, the `JAL` opcode, and bits 30:21 hold `0x24`, which is the offset
`0x48` moved down by a bit because the J format has no `imm[0]`. At `0x1000` that
offset means `0x1048`, which is the target the listing prints.

A branch is printed as the address it goes to rather than as the offset it
encodes, so that the targets can be read off the listing itself. `jalr` is the
exception, because its target depends on a register and is not known until it
runs.

Nothing is executed, so a listing covers the code a run never reaches: the
riscv-tests programs in `tests/` all start with a `jal` over their machine-mode
setup, which a `--debug` trace skips and a listing shows. The listing also names
the words a run would stop on, as `unimp`, which in a compiled program are the
zero words a linker pads the end of a segment with and the `wfi` the
riscv-tests trampoline spins on after the program has exited.

A listing and a trace are printed by the same code, so a line of one is a line of
the other.

The addresses in a listing are the ones the emulator itself uses, which for an
image loaded the way [below](#loading) it loads it are its file offsets rather
than the virtual addresses the ELF calls them. `0x1000` above is file offset
`0x1000`, which the ELF calls `0x80001000`.

## Interactive mode
To launch the emulator in interactive mode, pass the `-i` or `--interactive` option.

This mode currently supports 2 commands: 
* To see the contents of a register (decimal number):
    
    `reg 5`
* To see the contents of a memory location (physical address in hex):

    `mem 0123abcd`

An empty line executes the next instruction, and the instruction that was
executed is always printed. 
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

00001000:   0480006f          	jal     x0,00001048
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

00001048:   00000093           	addi    x1,x0,0
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

0000104c:   00000113           	addi    x2,x0,0
> 
```
## Using it as a library
The emulator is a library with a thin command line wrapper around it, so it can
be driven from Rust directly:

```rust
use rvemu::emulator::cpu::{Cpu, Outcome, RunConfig};

let mut cpu = Cpu::new(16);
cpu.load("tests/simple")?;
assert_eq!(cpu.run(&RunConfig::default()), Outcome::Exited(0));
```

## Loading
The whole file image is copied into memory at the addresses its file offsets
give it, which is the same as unpacking it segment by segment only when every
segment's file offset matches its virtual address. That holds for the programs
this emulator is meant to run, and it means a program built some other way will
not find its own code, its stack, or its data where it expects them.

## Writing a program the emulator can run
There is no kernel and no libc here, so a program is freestanding: it supplies
its own entry point and talks to the outside world through two system calls,
`write` and `exit`. There is no `printf`, but `write` is enough to print, and
the [example below](#a-program-that-prints) does.

### Get a RISC-V compiler
A bare-metal RISC-V GCC is what you want: either a
[riscv-gnu-toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
release, whose binary is called `riscv64-unknown-elf-gcc`, or one of the
[xPack builds](https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases),
which unpack into a directory you can add to `PATH` and call `riscv-none-elf-gcc`.
Everything below was run with the xPack GCC 15.2.0; the two names are the same
compiler, so substitute whichever you have.

### Compile and run
Save this as `fib.c`:

```c
/* fib.c -- exits with fib(10), which is 55. */
static int fib(int n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }

void _start(void)
{
    register int status asm("a0") = fib(10);
    register int number asm("a7") = 93; /* the exit system call */
    asm volatile("ecall" : : "r"(status), "r"(number) : "memory");
    __builtin_unreachable();
}
```

```
$ riscv-none-elf-gcc -O1 -march=rv32i -mabi=ilp32 \
      -nostdlib -nostartfiles -ffreestanding \
      -T rv32i.ld -o fib fib.c -lgcc
$ ./rvemu fib --stack
Program exited with exit code: 55
$ echo $?
55
```

Each flag matters:

- `-march=rv32i -mabi=ilp32` — the base integer set only. Anything beyond it,
  the compressed instructions in particular, will not decode.
- `-nostdlib` — no libc. No `printf`, no `malloc`; see the `write` example.
- `-nostartfiles` — no `crt0`, so nothing runs before `_start`.
- `-ffreestanding` — tells GCC not to assume the C library exists.
- `-lgcc` — **even with `-nostdlib`**, integer division and modulo still need
  GCC's runtime helpers (`__divsi3`, `__udivsi3`, ...), which live in libgcc.
  Without it a program using `/` or `%` fails to link with an undefined
  reference. Multiplication is fine: there is no `M` instruction for GCC to
  emit, so it expands `*` into shifts and adds.
- `-T rv32i.ld` — the linker script from this repository, explained below.

Add `-Wl,--no-warn-rwx-segments` if you want the linker to stop warning about
the script putting code and data in one loadable segment.

### A program that prints
`write` is the Linux system call 64, with the descriptor in `a0`, the address of
the bytes in `a1` and the count in `a2`. It returns the number of bytes written,
or -1 if the range is not inside the guest's memory. `exit` is 93, with the
status in `a0`.

```c
/* hello.c */
#define SYS_WRITE 64
#define SYS_EXIT  93

static long sys_write(int fd, const void *buf, unsigned long len)
{
    register long a0 asm("a0") = fd;
    register const void *a1 asm("a1") = buf;
    register long a2 asm("a2") = (long)len;
    register long a7 asm("a7") = SYS_WRITE;
    __asm__ volatile("ecall" : "+r"(a0) : "r"(a1), "r"(a2), "r"(a7) : "memory");
    return a0;
}

static void sys_exit(int code)
{
    register long a0 asm("a0") = code;
    register long a7 asm("a7") = SYS_EXIT;
    __asm__ volatile("ecall" : : "r"(a0), "r"(a7) : "memory");
    __builtin_unreachable();
}

void _start(void)
{
    const char *hello = "hello, world\n";
    unsigned n = 0;
    while (hello[n]) n++;
    if (sys_write(1, hello, n) < 0)
        sys_exit(1);
    sys_exit(0);
}
```

```
$ riscv-none-elf-gcc -O1 -march=rv32i -mabi=ilp32 \
      -nostdlib -nostartfiles -ffreestanding \
      -Wl,--no-warn-rwx-segments -T rv32i.ld -o hello hello.c -lgcc
$ ./rvemu hello --stack
hello, world
Program exited with exit code: 0
```

Wrapping that in a `puts` of your own is usually the first thing to do; `strlen`
is a loop, since there is no libc to call.

### `--stack` is not optional
`_start` runs with every register zeroed, so the stack pointer is 0 and the
first `sw` in a function prologue writes to address -4, which is not memory:

```
$ ./rvemu fib
thread 'main' panicked: range start index 4294967292 out of range
for slice of length 16384
```

`--stack` points the stack pointer at the top of memory, so a stack can grow
downwards into it. It applies the pointer once, before the program starts, so
deep recursion can still run off the bottom. The default memory is 16 KiB, which
is also the stack; raise it with `--mem` if the program needs more.

### Why the linker script
The emulator loads the whole file at its *file offsets* and translates the entry
point through whichever segment contains it. Code GCC generates reaches local
symbols pc-relatively, so it keeps working wherever it lands — which is why a
program with nothing but arithmetic runs under the default linker script. But
anything reached with an **absolute** address — a global, a string literal, a
`.bss` array — points at the wrong place, and the access lands outside memory:

```c
static unsigned table[8];
static const char name[] = "rvemu";
void _start(void) { /* sums table[] and name[] */ }
```

```
$ riscv-none-elf-gcc -O1 -march=rv32i ... -o globals globals.c   # no -T
$ ./rvemu globals --stack
thread 'main' panicked: range start index 4294965256 out of range
for slice of length 16384
```

`rv32i.ld` starts the image at 0x1000, which makes every virtual address the
same number as the file offset the loader uses for it. `.bss` has no file
content and is never loaded, which is fine: memory starts zeroed.

## Seeing what a program did
Beyond what it writes and the status it exits with — a status above 255 is
truncated to a byte — use `ebreak` as a breakpoint and watch the register file
as the program steps. This is the `globals.c` from above with an `ebreak` added
just before its `exit`:

```
$ ./rvemu globals --stack --interactive --registers
> 
  pc: 0x00001000
x0: 0x00000000    x1: 0x00000000    x2: 0x00003fff    x3: 0x00000000
...
> 
00001000:   00001737           	lui     x14,0x1
...
0000107c:   00100073           	ebreak
Program hit a breakpoint at 0x0000107c.
```

Put an `ebreak` in the program where you want to look:

```c
    asm volatile("ebreak");   /* a0 holds the checksum here */
```

`mem <hex address>` and `reg <number>` also work at the prompt, and `--debug`
traces every instruction as it retires. `ebreak` ends the run, so the register
file printed after each step is how you read a value out: by the time execution
reaches the breakpoint, `x10` above holds the checksum, and you can see it in the
dump printed on the step before.

Any system call number other than `write` and `exit` stops the run with
`Unimplemented ECALL`. There is no `read`, so a program cannot ask for input,
and stderr is not kept apart from stdout.

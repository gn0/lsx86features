
# `lsx86features`: list x86 extension sets used by a compiled binary

This CLI tool is useful for checking whether your compiled binaries make use of certain vector instructions that are not commonly available but your CPU supports.
It is essentially a wrapper around the [iced-x86](https://crates.io/crates/iced-x86) and [goblin](https://crates.io/crates/goblin) Rust crates.
If you are interested in how specific functions were compiled, you can also list instruction sets for specific symbol names.

For example, suppose that your CPU supports the [AVX-512](https://en.wikipedia.org/wiki/AVX-512) vector extension set:

```
$ lscpu | grep ^Flags | tr ' ' '\n' | grep avx512
avx512f
avx512dq
avx512ifma
avx512cd
avx512bw
avx512vl
avx512vbmi
avx512_vbmi2
avx512_vnni
avx512_bitalg
avx512_vpopcntdq
```

If you want to know whether functions in your hot path use this extension set, you can check and confirm with the following command:

```
$ lsx86features -si -F 'avx*' demo-asm/demo
            Function                Extension          Opcode      Count 
-------------------------------- ---------------- ---------------- ------
add_arrays_avx2                  avx              vaddps                1
add_arrays_avx2                  avx              vmovaps               3
add_arrays_avx2                  avx              vzeroall              1
add_arrays_avx512                avx              vzeroall              1
add_arrays_avx512                avx512f          vaddps                1
add_arrays_avx512                avx512f          vmovaps               3
```

## Installation

To install `lsx86features`, run

```
$ cargo install --locked --git https://github.com/gn0/lsx86features.git
```

## More examples

List all known instruction set extensions that are used by a compiled binary:

```
$ lsx86features demo-asm/demo | tail -n +3 | cut -f1 | uniq
avx
avx512f
cet_ibt
intel386
intel8086
multibytenop
sse
sse2
x64
```

List instructions in a compiled binary:

```
$ lsx86features demo-asm/demo | tail -n +3 | awk '{print $2}' | sort | uniq -c
      1 add
      1 addps
      1 and
      1 call
      3 cmp
...
      2 test
      2 vaddps
      2 vmovaps
      1 vzeroall
      1 xor
```

## To do

+ [ ] Add support for `.dynsym` so that shared libraries can be inspected, too.
+ [X] Implement JSON output.
+ [X] Resize header in the output according to maximum cell width.
+ [X] Demangle symbol names for C++ and Rust.
+ [X] Clean up the CLI.

## License

lsx86features is distributed under the GNU General Public License (GPL), version 3.
See the file [LICENSE](./LICENSE) for more information.


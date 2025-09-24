
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

If you want to know whether functions in your hot path use this extension set, you can check and confirm with `lsx86features`:

|   | for full binary | by function |
|---|-----------------|-------------|
| output as list | <img src="https://raw.githubusercontent.com/gn0/lsx86features/main/examples/output_l.png" width="250" alt="Output of `lsx86features -l demo-asm/demo`" /> | <img src="https://raw.githubusercontent.com/gn0/lsx86features/main/examples/output_ldF.png" width="250" alt="Output of `lsx86features -ldF 'ss*e*,avx*' demo-asm/demo`" /> |
| output as table | <img src="https://raw.githubusercontent.com/gn0/lsx86features/main/examples/output_tF.png" width="250" alt="Output of `lsx86features -tF 'ss*e*,avx*' demo-asm/demo`" /> | <img src="https://raw.githubusercontent.com/gn0/lsx86features/main/examples/output_tdF.png" width="250" alt="Output of `lsx86features -tdF 'ss*e*,avx*' demo-asm/demo`" /> |
| output as JSON | <img src="https://raw.githubusercontent.com/gn0/lsx86features/main/examples/output_jF.png" width="250" alt="Output of `lsx86features -jF 'ss*e*,avx*' demo-asm/demo`" /> | <img src="https://raw.githubusercontent.com/gn0/lsx86features/main/examples/output_jdF.png" width="250" alt="Output of `lsx86features -jdF 'ss*e*,avx*' demo-asm/demo`" /> |

## Features

| Feature | CLI option |
|---------|------------|
| List extension sets for each function. | `-s` or `--show-symbol` |
| Demangle symbol names for C++, Rust, and Swift. | `-d` or `--show-demangled` |
| Structured output as JSON. | `-j` or `--json` |
| Filter for extension sets (with wildcard support). | `-F` or `--feature-filter <STRING>` |
| Filter for function names (with wildcard support). | `-D` or `--demangled-symbol-filter <STRING>` |

## Installation

You can install `lsx86features` from crates.io:

```sh
cargo install --locked lsx86features
```

Or via this repository:

```sh
cargo install --locked --git https://github.com/gn0/lsx86features.git
```

## License

lsx86features is distributed under the GNU General Public License (GPL), version 3.
See the file [LICENSE](./LICENSE) for more information.


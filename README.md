
# `lsx86features`: list x86 extension sets used by a compiled binary

This program is useful for checking whether your compiled binaries make use of certain vector instructions that are not commonly available but your CPU supports.

For example, your CPU might support the [AVX-512](https://en.wikipedia.org/wiki/AVX-512) vector extension set.
If you want to know whether functions in your hot path use this extension set, you can check with the following command and confirm that it does not:

```
$ lsx86features -s your_binary
Functions that use the mmx extension set:
- err@@Base
- err@@Base-0x5ade0
- error@@Base
- getopt_long_only@@Base
- re_compile_fastmap@@Base
- re_match_2@@Base
- re_set_registers@@Base
- regcomp@@Base
- regerror@@Base
- regfree@@Base
- xrealloc@@Base

Functions that use the sse extension set:
- err@@Base
- err@@Base-0x5ade0
- error@@Base
- getopt_long_only@@Base
- regerror@@Base
- regfree@@Base
- xrealloc@@Base

Functions that use the sse2 extension set:
- err@@Base
- err@@Base-0x5ade0
- error@@Base
- getopt_long_only@@Base
- re_match_2@@Base
- regfree@@Base
- xrealloc@@Base
```

## Installation

`lsx86features` requires the following packages on Debian/Ubuntu:

- `binutils`
- `perl`

To install these, run

```
$ sudo apt install perl binutils
```

To install `lsx86features` in your user-specific `bin` directory, run

```
$ git clone https://codeberg.org/gnyeki/lsx86features
$ cd lsx86features
$ make install
```

## More examples

List all known instruction set extensions that are used by a compiled binary:

```
$ lsx86features your_binary | cut -f1 | sort | uniq
UNKNOWN
avx
avx2
mmx
sse
sse2
```

List instructions in a compiled binary:

```
$ lsx86features your_binary | sort -r | head -25
sse	924	movaps
sse	39	xorps
sse	2	movlps
sse	15	pmovmskb
sse	1242	movups
sse	1	movmskps
avx2	98	vpermpd
avx2	9	vpbroadcastd
avx2	8	vbroadcasti128
avx2	40	vpermq
avx2	4	vpsllvq
avx2	4	vinserti128
avx2	34	vpbroadcastb
avx2	26	vextracti128
avx2	16	vpermps
avx2	140	vperm2i128
avx2	14	vpbroadcastq
avx2	10	vpblendd
avx	7	vbroadcastss
avx	4	vextractf128
avx	34	vperm2f128
avx	1551	vzeroupper
avx	13	vpermilps
UNKNOWN	9972	jne
UNKNOWN	993	movl
```


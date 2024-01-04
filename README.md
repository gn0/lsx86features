
# `lsx86features`: list x86 features used by a compiled binary

Requires the following packages on Debian/Ubuntu:

- `binutils`
- `perl`

Example usage:

```
$ perl lsx86features.pl some_binary | sort -r | head -25
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


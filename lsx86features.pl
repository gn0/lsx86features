use strict;
use warnings;
use List::Util qw(any);

if ($#ARGV != 0) {
    die "Error: Must specify exactly one argument.\n\n",
      "Usage: perl lsx86features.pl <binary_filename>\n";
}

my $filename = $ARGV[0];

open(
    my $objdump_output,
    "-|",
    "objdump --no-show-raw-insn -d ${filename}"
) or die $!;

my %sets = (
    "mmx" => [
        qw(movd movq packssdw packsswb packuswb punpckhbw punpckhdq
           punpckhwd punpcklbw punpckldq punpcklwd paddb paddd paddsb
           paddsw paddusb paddusw paddw pmaddwd pmulhw pmullw psubb
           psubd psubsb psubsw psubusb psubusw psubw pcmpeqb pcmpeqd
           pcmpeqw pcmpgtb pcmpgtd pcmpgtw pand pandn por pxor pslld
           psllq psllw psrad psraw psrld psrlq psrlw emms)
    ],
    "sse" => [
        qw(movss movaps movups movlps movhps movlhps movhlps movmskps
           addss subss mulss divss rcpss sqrtss maxss minss rsqrtss
           addps subps mulps divps rcpps sqrtps maxps minps rsqrtps
           cmpss comiss ucomiss cmpps shufps unpckhps unpcklps cvtsi2ss
           cvtss2si cvttss2si cvtpi2ps cvtps2pi cvttps2pi andps orps
           xorps andnps pmulhuw psadbw pavgb pavgw pmaxub pminub pmaxsw
           pminsw pextrw pinsrw pmovmskb pshufw ldmxcsr stmxcsr movntq
           movntps maskmovq prefetch0 prefetch1 prefetch2 prefetchnta
           sfence)
    ],
    "sse2" => [
        qw(movapd movhpd movlpd movmskpd movsd movupd addpd addsd divpd
           divsd maxpd maxsd minpd minsd mulpd mulsd sqrtpd sqrtsd subpd
           subsd andnpd andpd orpd xorpd cmppd cmpsd comisd ucomisd
           shufpd unpckhpd unpcklpd cvtdq2pd cvtpd2dq cvtpd2pi cvtpd2ps
           cvtpi2pd cvtps2pd cvtsd2si cvtsd2ss cvtsi2sd cvtss2sd
           cvttpd2dq cvttpd2pi cvttsd2si cvtdq2ps cvtps2dq cvttps2dq
           movdq2q movdqa movdqu movq2dq paddq pmuludq pshufd pshufhw
           pshuflw pslldq psrldq psubq punpckhqdq punpcklqdq clflush
           lfence maskmovdqu mfence movntdq movnti movntpd pause)
    ],
    "sse3" => [
        qw(addsubpd addsubps haddpd haddps hsubpd hsubps lddqu movddup
           movshdup movsldup fisttp monitor mwait)
    ],
    "ssse3" => [
        qw(psignb psignw psignd pabsb pabsw pabsd palignr pshufb
           pmulhrsw pmaddubsw phsubw phsubd phsubsw phaddw phaddd
           phaddsw)
    ],
    "sse4" => [
        qw(mpsadbw phminposuw pmuldq pmulld dpps dppd blendps blendpd
           blendvps blendvpd pblendvb pblendw pminsb pmaxsb pminuw
           pmaxuw pminud pmaxud pminsd pmaxsd roundps roundss roundpd
           roundsd insertps pinsrb pinsrd pinsrq extractps pextrb
           pextrd/pextrq pmovsxbw pmovzxbw pmovsxbd pmovzxbd pmovsxbq
           pmovzxbq pmovsxwd pmovzxwd pmovsxwq pmovzxwq pmovsxdq
           pmovzxdq ptest pcmpeqq packusdw movntdqa)
    ],
    # String and text new instructions.
    "sse4sttni" => [
        qw(crc32 pcmpestri pcmpestrm pcmpistri pcmpistrm pcmpgtq)
    ],
    # AMD-specific.
    "sse4a" => [
        qw(extrq insertq movntsd movntss)
    ],
    "avx" => [
        qw(vbroadcastss vbroadcastsd vbroadcastf128
           vinsertf128
           vextractf128
           vmaskmovps vmaskmovpd
           vpermilps vpermilpd
           vperm2f128
           vtestps vtestpd
           vzeroall
           vzeroupper)
    ],
    "avx2" => [
        qw(vbroadcastss vbroadcastsd vpbroadcastb vpbroadcastw
           vpbroadcastd vpbroadcastq vbroadcasti128 vinserti128
           vextracti128 vgatherdpd vgatherqpd vgatherdps vgatherqps
           vpgatherdd vpgatherdq vpgatherqd vpgatherqq vpmaskmovd
           vpmaskmovq vpermps vpermd vpermpd vpermq vperm2i128 vpblendd
           vpsllvd vpsllvq vpsrlvd vpsrlvq vpsravd)
    ],
    "avx512f" => [
        qw(vpmovqd vpmovsqd vpmovusqd vpmovqw vpmovsqw vpmovusqw vpmovqb
           vpmovsqb vpmovusqb vpmovdw vpmovsdw vpmovusdw vpmovdb
           vpmovsdb vpmovusdb
           vcvtps2udq vcvtpd2udq vcvttps2udq vcvttpd2udq
           vcvtss2usi vcvtsd2usi vcvttss2usi vcvttsd2usi
           vcvtudq2ps vcvtudq2pd
           vcvtusi2ps vcvtusi2pd
           vcvtusi2sd vcvtusi2ss
           vcvtqq2pd vcvtqq2ps
           vpabsq
           vpmaxsq vpmaxuq
           vpminsq vpminuq
           vprold vprolvd vprolq vprolvq vprord vprorvd vprorq vprorvq
           vpscatterdd vpscatterdq vpscatterqd vpscatterqq
           vscatterdps vscatterdpd vscatterqps vscatterqpd)
    ],
    # Doubleword and quadword.
    "avx512dq" => [
        qw(vcvtps2qq vcvtpd2qq vcvtps2uqq vcvtpd2uqq vcvttps2qq
           vcvttpd2qq
           vcvttps2uqq vcvttpd2uqq vcvtuqq2ps vcvtuqq2pd
           vfpclassps vfpclasspd
           vfpclassss vfpclasssd
           vrangeps vrangepd
           vrangess vrangesd
           vreduceps vreducepd
           vreducess vreducesd
           vpmovm2d vpmovm2q
           vpmovd2m vpmovq2m
           vpmullq)
    ],
    # Byte and word.
    "avx512bw" => [
        qw(vpmovwb vpmovswb vpmovuswb
           vpmovm2b vpmovm2w
           vpmovb2m vpmovw2m)
    ],
    # Conflict detection.
    "avx512cd" => [
        qw(vpconflictd vpconflictq vplzcntd vplzcntq vpbroadcastmb2q
           vpbroadcastmw2d)
    ],
    # Exponential and reciprocal.
    "avx512er" => [
        qw(vexp2pd vexp2ps vrcp28pd vrcp28ps vrcp28sd vrcp28ss
           vrsqrt28pd vrsqrt28ps vrsqrt28sd vrsqrt28ss)
    ],
    # Prefetch.
    "avx512pf" => [
        qw(vgatherpf0dps vgatherpf0qps vgatherpf0dpd vgatherpf0qpd
           vgatherpf1dps vgatherpf1qps vgatherpf1dpd vgatherpf1qpd
           vscatterpf0dps vscatterpf0qps vscatterpf0dpd vscatterpf0qpd
           vscatterpf1dps vscatterpf1qps vscatterpf1dpd vscatterpf1qpd)
    ],
    "avx512vbmi2" => [
        qw(vpcompressb vpcompressw vpexpandb vpexpandw vpshld vpshldv
           vpshrd vpshrdv)
    ],
    # Vector neural network instructions.
    "avx512vnni" => [
        qw(vpdpbusd vpdpbusds vpdpwssd vpdpwssds)
    ],
    "avx512ifma" => [
        qw(vpmadd52luq vpmadd52huq)
    ],
    "avx512bitalg" => [
        qw(vpopcntb vpopcntw vpshufbitqmb)
    ],
    "avx512vpopcntdq" => [
        qw(vpopcntd vpopcntq)
    ],
    "avx512vp2intersect" => [
        qw(vp2intersectd vp2intersectq)
    ],
    # Galois field new instructions.
    "avx512gfni" => [
        qw(vgf2p8affineinvqb vgf2p8affineqb vgf2p8mulb)
    ],
    # Carry-less multiplication quadword.
    "avx512vpclmulqdq" => [
        qw(vpclmulqdq)
    ],
    "avx512vaes" => [
        qw(vaesdec vaesdeclast vaesenc vaesenclast)
    ],
    "avx512bf16" => [
        qw(vcvtne2ps2bf16 vcvtneps2bf16 vdpbf16ps)
    ],
    "avx512fp16" => [
        qw(vaddph vaddsh vsubph vsubsh vmulph vmulsh vdivph vdivsh
           vsqrtph vsqrtsh vfmadd132ph vfmadd213ph vfmadd231ph
           vfmadd132sh vfmadd213sh vfmadd231sh vfnmadd132ph vfnmadd213ph
           vfnmadd231ph vfnmadd132sh vfnmadd213sh vfnmadd231sh
           vfmsub132ph vfmsub213ph vfmsub231ph vfmsub132sh vfmsub213sh
           vfmsub231sh vfnmsub132ph vfnmsub213ph vfnmsub231ph
           vfnmsub132sh vfnmsub213sh vfnmsub231sh vfmaddsub132ph
           vfmaddsub213ph vfmaddsub231ph vfmsubadd132ph vfmsubadd213ph
           vfmsubadd231ph vreduceph vreducesh vrndscaleph vrndscalesh
           vscalefph vscalefsh vfmulcph vfmulcsh vfcmulcph vfcmulcsh
           vfmaddcph vfmaddcsh vfcmaddcph vfcmaddcsh vrcpph vrcpsh
           vrsqrtph vrsqrtsh vcmpph vcmpsh vcomish vucomish vmaxph
           vmaxsh vminph vminsh vfpclassph vfpclasssh vcvtw2ph vcvtuw2ph
           vcvtdq2ph vcvtudq2ph vcvtqq2ph vcvtuqq2ph vcvtps2phx
           vcvtpd2ph vcvtsi2sh vcvtusi2sh vcvtss2sh vcvtsd2sh vcvtph2w
           vcvttph2w vcvtph2uw vcvttph2uw vcvtph2dq vcvttph2dq
           vcvtph2udq vcvttph2udq vcvtph2qq vcvttph2qq vcvtph2uqq
           vcvttph2uqq vcvtph2psx vcvtph2pd vcvtsh2si vcvttsh2si
           vcvtsh2usi vcvttsh2usi vcvtsh2ss vcvtsh2sd vgetexpph
           vgetexpsh vgetmantph vgetmantsh vmovsh vmovw)
    ],
);

my %instructions = ();

sub find_set {
    my ($instruction) = @_;

    for my $instruction_set (keys %sets) {
        if (any { $_ eq $instruction } @{$sets{$instruction_set}}) {
            return $instruction_set;
        }
    }

    return undef;
}

while (my $line = <$objdump_output>) {
    if ($line !~ /^\s+[0-9a-f]+:\t(\w+)/) {
        next;
    }

    if (not exists $instructions{$1}) {
        $instructions{$1} = 1;
    } else {
        $instructions{$1} += 1;
    }
}

for my $instruction (keys %instructions) {
    my $instruction_set = find_set($instruction) // "UNKNOWN";

    print
        "${instruction_set}\t",
        "${instructions{$instruction}}\t",
        "${instruction}\n";
}

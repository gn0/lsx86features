    global  add_arrays_avx512

    section .text

add_arrays_avx512:
    ; Prologue.
    ;push    rdi
    ;sub     rsp, 20
    ;mov     rdi, rsp
    vzeroall

    ; Function body.
    ;vmovaps zmm0, zmmword ptr [rdx]
    ;vmovaps zmm1, zmmword ptr [r8]
    vmovaps zmm0, [rsi]
    vmovaps zmm1, [rdx]
    vaddps  zmm2, zmm0, zmm1
    ;vmovaps zmmword ptr[rcx], zmm2
    vmovaps [rdi], zmm2

    ; Epilogue.
    ;add     rsp, 20
    ;pop     rdi
    ret

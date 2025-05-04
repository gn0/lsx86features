    global  add_arrays_avx2

    section .text

add_arrays_avx2:
    ; Prologue.
    vzeroall

    ; Function body.
    vmovaps ymm0, [rsi]
    vmovaps ymm1, [rdx]
    vaddps  ymm2, ymm0, ymm1
    vmovaps [rdi], ymm2

    ; Epilogue.
    ret

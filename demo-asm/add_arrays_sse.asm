    global  add_arrays_sse

    section .text

add_arrays_sse:
    ; Prologue.

    ; Function body.
    movaps  xmm0, [rsi]
    movaps  xmm1, [rdx]
    addps   xmm0, xmm1
    movaps  [rdi], xmm0

    ; Epilogue.
    ret

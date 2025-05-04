#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern void add_arrays_sse(float result[], float x_1[], float x_2[]);
extern void add_arrays_avx2(float result[], float x_1[], float x_2[]);
extern void add_arrays_avx512(float result[], float x_1[], float x_2[]);

int main(int argc, char *argv[]) {
    float __attribute__ ((aligned (64))) x_1[] = {
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
    };
    float __attribute__ ((aligned (64))) x_2[] = {
        2, 1, 4, 3, 6, 5, 8, 7, 10, 9, 12, 11, 14, 13, 16, 15
    };

    float __attribute__ ((aligned (64))) result[16] = {0};

    if (argc <= 1 || strcmp(argv[1], "sse") == 0) {
        add_arrays_sse(result, x_1, x_2);
        add_arrays_sse(&result[4], &x_1[4], &x_2[4]);
        add_arrays_sse(&result[8], &x_1[8], &x_2[8]);
        add_arrays_sse(&result[12], &x_1[12], &x_2[12]);
    } else if (strcmp(argv[1], "avx2") == 0) {
        add_arrays_avx2(result, x_1, x_2);
        add_arrays_avx2(&result[8], &x_1[8], &x_2[8]);
    } else if (strcmp(argv[1], "avx512") == 0) {
        add_arrays_avx512(result, x_1, x_2);
    } else {
        errx(EXIT_FAILURE, "invalid algorithm: %s", argv[1]);
    }

    for (size_t i = 0; i < 16; i++) {
        printf(
            "result[%ld] = %.02f == %ld\n",
            i, result[i], 2 * (i / 2) + 3
        );
    }

    return 0;
}

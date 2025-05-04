.PHONY: build
build: demo

demo: main.o add_arrays_sse.o add_arrays_avx2.o add_arrays_avx512.o
	gcc -g $^ -o $@ -Wall

main.o: main.c
	gcc -g $< -c -o $@ -Wall

add_arrays_%.o: add_arrays_%.asm
	nasm -g -felf64 $<

.PHONY: clean
clean:
	-rm *.o demo

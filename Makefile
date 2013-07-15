all: malloc
debug: malloc-debug

malloc:
	rustc malloc.rs -O -o malloc

malloc-debug:
	rustc --cfg debug malloc.rs -o malloc-debug

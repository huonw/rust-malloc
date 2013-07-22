all: malloc mmap
debug: malloc-debug mmap-debug

# it's quick to compile, so don't bother with being fine-grained about this.
malloc: *.rs
	rustc malloc.rs -O -o malloc

malloc-debug: *.rs
	rustc --cfg debug malloc.rs -o malloc-debug

mmap: *.rs
	rustc mmap.rs -O -o mmap
mmap-debug: *.rs
	rustc --cfg debug mmap.rs -o mmap-debug

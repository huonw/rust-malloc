all: malloc
debug: malloc-debug

# it's quick to compile, so don't bother with being fine-grained about this.
malloc: *.rs
	rustc malloc.rs -O -o malloc

malloc-debug: *.rs
	rustc --cfg debug malloc.rs -o malloc-debug

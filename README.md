# Rust malloc

A pure-Rust proof-of-concept implementation of `malloc`/`free` for
x86-64 Linux. It is very, very basic: a linked list of allocations
(traversed on every `malloc` call!), first-fit strategy and all
allocations come directly from the `brk` syscall. As such, it is very
slow, not-threadsafe and generally bad. There is little-to-no error
checking.

Uses a modified version of
[zero.rs](https://github.com/pcwalton/zero.rs) and a
[tiny syscall](https://gist.github.com/Aatch/5894562)
wrapper. Requires a recent `rustc` from master.

There is an example of a custom version of `~T` using this allocator
at the bottom of `malloc.rs`.

## TODO

- Use `mmap`.
- Return memory to the OS.
- Merge adjacent free cells better.

### Wishlist

- Make it fast.
- Support other platforms/operating systems.

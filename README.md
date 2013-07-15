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

Test with:

    make
    ./malloc

To enable the debugging `assert`s:

    make debug
    ./malloc-debug

Code map:

- `malloc.rs` contains `malloc`, `free`, etc, as well as `main` and some tests/examples.
- `types.rs` contains the allocation header (`Header`, surprisingly)
  as well some newtype structs around raw pointers (`Box` and `Data`;
  in theory these have no runtime cost over using `*mut Header` and
  `*mut u8` directly) which make the implementation relatively
  type-safe.
- `util.rs` & `diag.rs` contain utilities and tools for printing
  diagnostics about the current allocation state.

**Only useful for segfaulting and generally making programs incorrect.**

## TODO

- Use `mmap`.
- Return memory to the OS.
- Merge adjacent free cells better.

### Wishlist

- Make it fast.
- Support other platforms/operating systems.

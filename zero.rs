//
// zero.rs
//
// Minimal definitions of the core primitives in Rust. Include this file with
// your project to create a freestanding Rust program that can run on bare
// metal.
//

#[allow(ctypes, unused_variable)];

pub trait Drop {
    #[lang="drop"]
    fn drop(&self);
}

pub type c_char = u8;
pub type size_t = uint;

#[lang="fail_"]
pub fn fail_(expr: *c_char, file: *c_char, line: size_t) -> ! {
    unsafe{abort()}
}


#[lang="fail_bounds_check"]
pub fn fail_bounds_check(file: *c_char, line: size_t,
                         index: size_t, len: size_t) {
    unsafe {abort()}
}

#[lang="start"]
pub fn start(main: *u8, _: int, _: **i8, _: *u8) -> int {
    unsafe {
        let main: extern "Rust" fn() = transmute(main);
        main();
        0
    }
}

extern {
    #[fast_ffi]
    pub fn abort() -> !;
}

// Rust intrinsic dependencies

extern "rust-intrinsic" {
    pub fn transmute<T,U>(val: T) -> U;
    pub fn size_of<T>() -> uint;
    pub fn forget<T>(val: T);
    pub fn move_val<T>(dst: &mut T, src: T);
    pub unsafe fn uninit<T>() -> T;
}


#[macro_escape];

macro_rules! print {
    ($a:expr, $b:expr) => {{
        ::util::puts($a);
        ::util::putn($b as uint);
        util::puts("\n");
    }}
}

/// Prints the message and exits.
pub fn fail(msg: &str) -> ! {
    puts(msg);
    puts("\n");
    unsafe { ::zero::abort(); }
}

/// Conditionally defining macros for --cfg debug.
#[macro_escape]
#[cfg(debug)]
pub mod debugging {
    macro_rules! assert {
        ($test:expr, $message:expr) => {
            if !$test {
                ::util::fail($message)
            }
        }
    }
}
#[macro_escape]
#[cfg(not(debug))]
mod debugging {
    macro_rules! assert { ($test:expr, $message:expr) => {{}} }
}

/// Print a string.
#[inline(never)]
pub fn puts(s: &str) {
    unsafe {
        let (x, len) = ::zero::transmute::<&str, (*u8, uint)>(s);
        write(1, x, len - 1);
    }
}
/// Print an integer.
#[inline(never)]
pub fn putn(mut x: uint) {
    let mut out = [' ' as u8, .. 20];
    let mut i = 19u;
    if x == 0 {
        out[i] = '0' as u8;
        i -= 1
    }
    while x != 0 {
        let digit = x % 10;
        out[i] = '0' as u8 + digit as u8;
        i -= 1;
        x /= 10;
    }

    unsafe { write(1, ((&out) as *[u8, .. 20] as uint + i + 1) as *u8, 19 - i); }
}

/// Round `n` up to the nearest power of two.
pub fn round_up(mut n: uint) -> uint {
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

extern {
    fn write(x: int, y: *u8, z: uint) -> int;
}

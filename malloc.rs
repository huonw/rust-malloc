#[allow(ctypes, unused_variable)];
#[no_std];mod zero;

macro_rules! print{($a:expr, $b:expr) => {{puts($a); putn($b as uint); puts("\n"); }}}

#[inline(never)]
fn puts(s: &str) {
    unsafe {
        let (x, len) = zero::transmute::<&str, (*u8, uint)>(s);
        write(1, x, len);
    }
}
#[inline(never)]
fn putn(mut x: uint) {
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

mod syscall;

static SC_BRK: int = 12;

struct Header {
    prev: *mut Header,
    next: *mut Header,
    size: uint,
    free: bool
}

static mut current_brk: *u8 = 0 as *u8;

#[no_mangle]
unsafe fn brk(ptr: *u8) -> *u8 {
    let new = syscall::syscall1(SC_BRK, ptr as int);
    current_brk = new as *u8;
    (if new < (ptr as int) {-1} else {new}) as *u8
}

unsafe fn sbrk(increment: int) -> *u8 {
    if current_brk as uint == 0 {
        brk(0 as *u8);
    }
    let new = current_brk as int + increment;
    brk(new as *u8)
}

static mut malloc_root: *mut Header = 0 as *mut Header;

pub fn round_up(mut n: uint) -> uint {
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

pub fn init_malloc() {
    unsafe {
        let header_size = zero::size_of::<Header>();

        // set it up.
        malloc_root = brk(0 as *u8) as *mut Header;
        sbrk(header_size as int);
        (*malloc_root) = Header {
            next: 0 as *mut Header,
            prev: 0 as *mut Header,
            size: 0,
            free: true
        }
    }
}

pub fn malloc(size: uint) -> *mut u8 {
    unsafe {
        let header_size = zero::size_of::<Header>();

        if current_brk as uint == 0 {
            init_malloc();
        }

        let rounded_size = round_up(size);
        let total_size = header_size + rounded_size;

        let mut ptr = malloc_root;
        while (*ptr).next as uint != 0 {
            if (*ptr).free && (*ptr).size >= rounded_size {
                if (*ptr).size <= total_size { // allocation fits perfectly
                    (*ptr).free = false;
                } else {
                    let next_header = (ptr as uint + rounded_size) as *mut Header;

                    (*next_header).prev = ptr;
                    (*next_header).next = (*ptr).next;
                    (*next_header).size = (*ptr).size - total_size;
                    (*next_header).free = true;

                    (*ptr).size = rounded_size;
                    (*ptr).free = false;
                    (*ptr).next = next_header;
                }
                return (ptr as uint + header_size) as *mut u8;
            }

            ptr = (*ptr).next;
        }

        // invariant: the last Header (`ptr` here) in the linked-list
        // is always free.

        let last_header = (sbrk(total_size as int) as uint - header_size) as *mut Header;
        (*ptr).next = last_header;
        (*ptr).size = rounded_size;
        (*ptr).free = false;

        (*last_header).prev = ptr;
        (*last_header).size = 0;
        (*last_header).free = true;

        // shift to point beyond the last byte
        (ptr as uint + header_size) as *mut u8
    }
}

pub fn realloc(ptr: *mut u8, size: uint) -> *mut u8 {
    0 as *mut u8
}

pub fn calloc(size: uint) -> *mut u8 {
    let ptr = malloc(size);
    let mut idx = ptr;
    let end = ptr as uint + size;
    while (idx as uint) < end {
        unsafe { *idx = 0; }
        idx = (idx as uint + 1) as *mut u8;
    }
    ptr
}

pub fn free(ptr: *mut u8) {
    unsafe {
        let header_size = zero::size_of::<Header>();
        let block = (ptr as uint - header_size) as *mut Header;

        (*block).free = true;
    }
}

pub fn count_blocks() -> (uint, uint, uint, uint) {
    unsafe {
        let mut ptr = malloc_root;
        let mut free = 0;
        let mut fsize = 0;
        let mut not_free = 0;
        let mut nsize = 0;

        while ptr as uint != 0 {
            if (*ptr).free {
                free += 1;
                fsize += (*ptr).size;
            } else {
                not_free += 1;
                nsize += (*ptr).size;
            }

            ptr = (*ptr).next;
        }

        (free, fsize, not_free, nsize)
    }
}

pub fn diagnostics() {
    let (f, fs, n, ns) = count_blocks();

    puts(  "\t** DIAGS **\n");
    print!("\t*    f = ", f);
    print!("\t*  f s = ", fs);
    print!("\t*   nf = ", n);
    print!("\t* nf s = ", ns);
}


fn main() {
    unsafe {
        let mut i = 1000;
        while i > 0 {
            print!("i = ", i);
            let x = malloc(i) as *mut uint;
            print!("x = ", x);
            *x = 10;
            diagnostics();
            free(x as *mut u8);
            diagnostics();

            i -= 100;
        }

        puts("Interleaved. Allocating...\n");
        let x = malloc(10000);
        diagnostics();
        let y = malloc(10000);
        diagnostics();
        let z = malloc(10000);
        diagnostics();
        puts("Freeing...\n");
        free(z);
        diagnostics();
        free(x);
        diagnostics();
        free(y);
        diagnostics();
    }
}


extern {
    fn write(x: int, y: *u8, z: uint) -> int;
}

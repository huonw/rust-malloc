#[allow(ctypes, unused_variable)];
#[no_std];
mod zero;

mod util;
mod syscall;

static SC_BRK: int = 12;

struct Header {
    prev: Box,
    next: Box,
    size: uint,
    free: bool
}

impl Header {
    fn default() -> Header {
        Header {
            prev: Box::null(),
            next: Box::null(),
            size: 0,
            free: true
        }
    }
}


#[inline(always)]
fn header_size() -> uint {
    // function rather than macro because a macro complains about
    // unnecessary `unsafe` if used inside an `unsafe` block
    unsafe { ::zero::size_of::<Header>() }
}

// (header, data...)
struct Box(*mut Header);

impl Box {
    #[inline(always)]
    fn null() -> Box { Box::from_uint(0) }

    #[inline(always)]
    fn from_ptr(ptr: *mut u8) -> Box { Box(ptr as *mut Header) }
    #[inline(always)]
    fn from_uint(u: uint) -> Box { Box(u as *mut Header) }

    #[inline]
    fn data(&self) -> Data {
        Data::from_uint((**self as uint) + header_size())
    }

    /// Returns the box that fits immediately after this allocation.
    #[inline]
    fn next_box_by_size(&self) -> Box {
        // 3-star programming!
        Box::from_uint((**self) as uint + header_size() + self.size())
    }

    /// Splits a box into a non-free and free section, updating the
    /// linked list appropriately. This doesn't check that the
    /// `data_size` argument is sensible (i.e. can fit within the self
    /// box).
    fn split_box(&self, data_size: uint) -> Box {
        let old_size = self.size();

        assert!(old_size != 0 || !self.has_next(),
                "Calling split_box on an empty within the list");
        assert!(old_size == 0 || data_size + header_size() < old_size,
                "Calling split_box without enough space");

        self.header().size = data_size;

        let new_box = self.next_box_by_size();

        // update the back links
        if self.has_next() {
            self.next().header().prev = new_box;
        }

        *new_box.header() = Header {
            prev: *self,
            next: self.next(),
            // If the current box is 0 sized and we're calling split,
            // then we're probably at the end, so the trailing one
            // should also be 0 sized. If the current box has space,
            // then subtract off size of data, and size of this header
            // to work out how much remains.
            size: if old_size == 0 {0} else {old_size - data_size - header_size()},
            free: true
        };

        self.header().next = new_box;
        self.header().free = false;

        new_box
    }

    #[inline]
    fn next(&self) -> Box {
        self.header().next
    }
    #[inline]
    fn has_next(&self) -> bool {
        *self.next() as uint != 0
    }

    #[inline]
    fn prev(&self) -> Box {
        self.header().prev
    }
    #[inline]
    fn size(&self) -> uint {
        self.header().size
    }
    #[inline]
    fn is_free(&self) -> bool {
        self.header().free
    }

    #[inline(always)]
    fn header<'a>(&'a self) -> &'a mut Header {
        unsafe { &mut ***self }
    }
}

// [header](data...)
struct Data(*mut u8);

impl Data {
    #[inline(always)]
    fn from_uint(u: uint) -> Data { Data(u as *mut u8) }
    #[inline(always)]
    fn box(&self) -> Box {
        Box::from_uint((**self as uint) - header_size())
    }
}


static mut current_brk: *mut u8 = 0 as *mut u8;

unsafe fn brk(ptr: *mut u8) -> *mut u8 {
    let new = syscall::syscall1(SC_BRK, ptr as int);
    current_brk = new as *mut u8;
    (if new < (ptr as int) {-1} else {new}) as *mut u8
}

unsafe fn sbrk(increment: int) -> *mut u8 {
    if current_brk as uint == 0 {
        brk(0 as *mut u8);
    }
    let new = current_brk as int + increment;
    brk(new as *mut u8)
}

static mut malloc_root: Box = Box(0 as *mut Header);

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
        malloc_root = Box(brk(0 as *mut u8) as *mut Header);
        sbrk(header_size as int);
        (**malloc_root) = Header::default()
    }
}

pub fn boxy_malloc(size: uint) -> Box {
    unsafe {
        if current_brk as uint == 0 {
            init_malloc();
        }

        let total_size = round_up(header_size() + size);
        // allocate as much as possible for the data
        let data_size = total_size - header_size();

        let mut ptr = malloc_root;
        while ptr.has_next() {
            if ptr.is_free() && ptr.size() >= size {
                ptr.header().free = false;
                if ptr.size() > total_size {
                    // there is space more than just the data (in fact
                    // there is space for at least the data and
                    // another header), so split this box into two
                    // subboxes.
                    ptr.split_box(data_size);
                }
                return ptr;
            }

            ptr = ptr.next();
        }

        let new_ptr = sbrk(total_size as int);
        let new_box = Box::from_ptr(new_ptr);
        *new_box.header() = Header {
            next: Box::null(),
            prev: ptr,
            size: data_size,
            free: false
        };
        ptr.header().next = new_box;
        new_box
    }
}

pub fn malloc(size: uint) -> *mut u8 {
    *boxy_malloc(size).data()
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
    let ptr = Data(ptr);
    ptr.box().header().free = true;
}

pub fn count_blocks() -> (uint, uint, uint, uint) {
    unsafe {
        let mut ptr = malloc_root;
        let mut free = 0;
        let mut fsize = 0;
        let mut not_free = 0;
        let mut nsize = 0;

        while ptr.has_next() {
            if ptr.is_free() {
                free += 1;
                fsize += ptr.size();
            } else {
                not_free += 1;
                nsize += ptr.size();
            }

            ptr = ptr.next();
        }

        (free, fsize, not_free, nsize)
    }
}

pub fn diagnostics() {
    let (f, fs, n, ns) = count_blocks();

    util::puts(  "\t** DIAGS **\n");
    print!("\t*    f = ", f);
    print!("\t*  f s = ", fs);
    print!("\t*   nf = ", n);
    print!("\t* nf s = ", ns);
}


fn main() {
    unsafe {
        //general_test();
        basic_bench();
    }
}
unsafe fn general_test() {
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

    util::puts("Interleaved. Allocating...\n");
    let x = malloc(10000);
    diagnostics();
    let y = malloc(10000);
    diagnostics();
    let z = malloc(10000);
    diagnostics();
    util::puts("Freeing...\n");
    free(z);
    diagnostics();
    free(x);
    diagnostics();
    free(y);
    diagnostics();
}
unsafe fn basic_bench() {
    static LIMIT: uint = 10000000;
    let mut i = 0;
    while i < LIMIT {
        let size = 100 + ((i*i - 13) * 32313) % 100000;
        free(malloc(size));
        //zero::free(zero::malloc(size));
        if i % (LIMIT / 5) == 0 {
            print!("i = ", i);
            diagnostics();
        }
        i += 1;
    }
}


extern {
    fn write(x: int, y: *u8, z: uint) -> int;
}

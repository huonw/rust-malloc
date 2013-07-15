#[allow(ctypes, unused_variable)];
#[no_std];
use types::{Header, Data, Box, header_size};

mod zero;

mod util;
mod syscall;
mod types;

static SC_BRK: int = 12;


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

pub fn init_malloc() {
    unsafe {
        // set it up.
        malloc_root = Box::from_ptr(brk(0 as *mut u8));
        sbrk(header_size() as int);
        *malloc_root.header() = Header::default();
    }
}

pub fn boxy_malloc(size: uint) -> Box {
    let mut prev = unsafe{malloc_root};
    if prev.is_null() {
        init_malloc();
        prev = unsafe{malloc_root};
    }
    let mut box = prev.next();

    let total_size = util::round_up(header_size() + size);
    // leave as much as possible for the data
    let data_size = total_size - header_size();

    // traverse the linked list of allocations, and take the first
    // free block that fits.
    while !box.is_null() {
        if box.is_free() && box.size() >= size {
            box.header().free = false;
            if box.size() > total_size {
                // there is space for at least the data and another
                // header, so split this box into two subboxes.
                box.split_box(data_size);
            }
            return box;
        }
        prev = box;
        box = box.next();
    }

    // nothing was large enough, so make a new block at the end.
    let current = unsafe{sbrk(0)};
    let new_ptr = unsafe{sbrk(total_size as int)};
    assert!(new_ptr as uint != 0, "new_ptr is null");

    let new_box = Box::from_ptr(current);
    assert!(!new_box.is_null(), "new_box is null");
    *new_box.header() = Header {
        next: Box::null(),
        prev: prev,
        size: data_size,
        free: false
    };
    prev.header().next = new_box;
    new_box
}

pub fn malloc(size: uint) -> *mut u8 {
    *boxy_malloc(size).data()
}

pub fn realloc(ptr: *mut u8, size: uint) -> *mut u8 {
    // who needs valid pointers anyway?
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
    // XXX: return memory to the OS.
    if ptr as uint == 0 { return; }

    let ptr = Data(ptr).box();

    ptr.header().free = true;

    // XXX: merge adjacent free blocks. Broken because of things.
    // attempt to merge with the next block (if it's free)
    // ptr.try_merge();

    // attempt to merge with the previous block (if it's free).
    // ptr.prev().try_merge();
}


pub fn print_boxes() {
    use util::{puts, putn};
    let mut ptr = unsafe{malloc_root};
    puts("\t++ CHAIN ++\n");
    while !ptr.is_null() {
        puts("\t+ ptr = ");
        putn(*ptr as uint);
        puts(", size = ");
        putn(ptr.size());
        puts(if ptr.is_free() {", free\n"} else {", alloc\n"});

        ptr = ptr.next();
    }
}

pub fn count_blocks() -> (uint, uint, uint, uint) {

    let mut count = 0;
    let mut ptr = unsafe {malloc_root};
    let mut free = 0;
    let mut fsize = 0;
    let mut not_free = 0;
    let mut nsize = 0;


    while !ptr.is_null() && count < 10_000_000 {
        if ptr.is_free() {
            free += 1;
            fsize += ptr.size();
        } else {
            not_free += 1;
            nsize += ptr.size();
        }

        ptr = ptr.next();
        // catch infinite loop bugs.
        count += 1;
    }

    assert!(ptr.is_null(), "More than 10000000 allocations.");

    (free, fsize, not_free, nsize)
}

pub fn diagnostics() {
    let (f, fs, n, ns) = count_blocks();

    util::puts(  "\t** DIAGS **\n");
    print!("\t*  free # = ", f);
    print!("\t*    free = ", fs);
    print!("\t* alloc # = ", n);
    print!("\t*   alloc = ", ns);
}


#[allow(unused_unsafe)]
fn main() {
    unsafe {
        //general_test();
        //basic_bench();
        //interleaved_bench();
    }
    unique_ptrs();
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
    print!("i = ", i);
    diagnostics();
}
unsafe fn interleaved_bench() {
    static LIMIT: uint = 10000;
    let mut ptrs = [0 as *mut u8, .. LIMIT];
    let mut i = 0;
    //malloc(1000);
    while i < LIMIT {
        let size = 100 + ((i*i - 13) * 32313) % 100000;
        ptrs[i] = malloc(size);
        //ptrs[i] = zero::malloc(size);
        if i % (LIMIT / 5) == 0 {
            print!("i = ", i);
            diagnostics();
        }
        i += 1;
    }
    print!("i = ", i);
    diagnostics();

    while i != 0 {
        let size = 100 + ((i*i - 13) * 32313) % 100000;
        free(ptrs[i - 1]);
        //zero::free(ptrs[i - 1] as *u8);
        if i % (LIMIT / 5) == 0 {
            print!("i = ", i);
            diagnostics();
        }
        i -= 1;
    }
    print!("i = ", i);
    diagnostics();
}

fn unique_ptrs() {
    use zero::{size_of, transmute, move_val, uninit, Drop};

    #[unsafe_no_drop_flag] // make this pointer one word.
    struct Unique<T> {
        priv ptr: *mut T
    }

    impl<T> Unique<T> {
        fn new(val: T) -> Unique<T> {
            unsafe {
                let ptr: *mut T = transmute(malloc(size_of::<T>()));
                *ptr = val;
                Unique { ptr: ptr }
            }
        }
    }

    #[unsafe_destructor]
    impl<T> Drop for Unique<T> {
        fn drop(&self) {
            print!("dropping unique, ptr = ", self.ptr);
            if self.ptr as uint != 0 {
                unsafe {
                    // run the inner dtor
                    move_val(&mut *self.ptr, uninit());

                    free(self.ptr as *mut u8);
                    // yuck
                    let mut_self: &mut Unique<T> = transmute(self);
                    mut_self.ptr = 0 as *mut T;
                }
            }
        }
    }

    struct TestDtor(int);
    impl Drop for TestDtor {
        fn drop(&self) {
            print!("dropping testdtor, val = ", **self);
        }
    }

    print!("size of unique = ", unsafe{size_of::<Unique<TestDtor>>()});

    diagnostics();
    {
        let t = Unique::new(TestDtor(123));

        let x = t;
        // let y = t; // error: use of moved value: `t`
        diagnostics();
    }
    diagnostics();
    {
        let a = Unique::new(0);
        let b = Unique::new(a);
        let c = Unique::new(b);
        diagnostics();
    }
    diagnostics();
}

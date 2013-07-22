#[no_std];
#[allow(ctypes, unused_variable)];

mod syscall;
mod util;
mod zero;

static SC_MMAP: int = 9;
static SC_MREMAP: int = 25;
static SC_MUNMAP: int = 11;

static PROT_READ: uint = 1;
static PROT_WRITE: uint = 2;

static MAP_SHARED: uint    = 0x01;
static MAP_PRIVATE: uint   = 0x02;
static MAP_FIXED: uint     = 0x10;
static MAP_ANONYMOUS: uint = 0x20;

unsafe fn mmap(addr: *mut u8, length: uint,
               prot: uint, flags: uint,
               fd: i32, offset: uint) -> *mut u8 {
    syscall::syscall6(SC_MMAP,
                      addr as int, length as int,
                      prot as int, flags as int,
                      fd as int, offset as int) as *mut u8
}
unsafe fn munmap(addr: *mut u8, length: uint) {
    syscall::syscall2(SC_MUNMAP,
                      addr as int, length as int);
}

fn malloc(size: uint) -> *mut u8 {
    unsafe {
        let uint_size =zero::size_of::<uint>();
        let ptr = mmap(0 as *mut u8,
                       uint_size + size,
                       PROT_READ | PROT_WRITE,
                       MAP_PRIVATE | MAP_ANONYMOUS,
                       -1 , 0);
        *(ptr as *mut uint) = size;
        (ptr as uint + uint_size) as *mut u8
    }
}
fn free(ptr: *mut u8) {
    unsafe {
        let uint_size =zero::size_of::<uint>();
        let ptr = (ptr as uint - uint_size) as *mut uint;
        munmap(ptr as *mut u8, *ptr);
    }
}


fn diagnostics() {}
#[allow(unused_unsafe)]
fn main() {
    unsafe {
        //general_test();
        basic_bench();
        //interleaved_bench();
    }
    //unique_ptrs();
}
unsafe fn general_test() {
    let mut i = 1000;
    diagnostics();
    let x = malloc(1);
    diagnostics();
    free(x);
    diagnostics();
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
            //print!("i = ", i);
            diagnostics();
        }
        i += 1;
    }
    //print!("i = ", i);
    diagnostics();
}
unsafe fn interleaved_bench() {
    static LIMIT: uint = 10000;
    let mut ptrs = [0 as *mut u8, .. LIMIT];
    let mut i = 0;
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

    // prints when dropped, for testing that Unique behaves correctly.
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
    } // should be freed here
    diagnostics();
    {
        let a = Unique::new(0);
        let b = Unique::new(a);
        let c = Unique::new(b);
        diagnostics();
    }
    diagnostics();
}

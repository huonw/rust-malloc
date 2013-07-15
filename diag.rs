use util::{puts, putn};

pub fn print_boxes() {
    let mut ptr = unsafe{super::malloc_root};
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
    let mut ptr = unsafe {super::malloc_root};
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

    puts(  "\t** DIAGS **\n");
    print!("\t*  free # = ", f);
    print!("\t*    free = ", fs);
    print!("\t* alloc # = ", n);
    print!("\t*   alloc = ", ns);
}

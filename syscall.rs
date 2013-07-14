// https://gist.github.com/Aatch/5894562

#[inline(always)]
pub unsafe fn syscall0(n: int) -> int {
    let mut ret : int = 0;

    asm!("syscall" : "=ra"(ret) : "ra"(n) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall1(n: int, a1: int) -> int {
    let mut ret : int = 0;

    asm!(
        "
movq $1, %rax
movq $2, %rdi
syscall
movq %rax, $0
": "=r"(ret): "r"(n), "r"(a1) : "rax", "rcx", "rdi", "r11", "memory" : "volatile"
    );
    //asm!("syscall" : "=ra"(ret) : "ra"(n), "rD"(a1) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall2(n: int, a1: int, a2: int) -> int {
    let mut ret : int = 0;

    asm!("syscall" : "=ra"(ret) : "0"(n), "rS"(a1), "rD"(a2) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall3(n: int, a1: int, a2: int, a3: int) -> int {

    let mut ret : int = 0;

    asm!("syscall" : "=ra"(ret) : "ra"(n), "rD"(a1), "rS"(a2), "rd"(a3) : "rcx", "r11", "memory" : "volatile");

    return ret;
}
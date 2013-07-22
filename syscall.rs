// https://gist.github.com/Aatch/5894562

#[inline(always)]
pub unsafe fn syscall0(n: int) -> int {
    let mut ret : int = 0;

    asm!("syscall" : "={rax}"(ret) : "{rax}"(n) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall1(n: int, a1: int) -> int {
    let mut ret : int = 0;

    asm!("syscall" : "={rax}"(ret) : "{rax}"(n), "{rdi}"(a1) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall2(n: int, a1: int, a2: int) -> int {
    let mut ret : int = 0;

    asm!("syscall" : "={rax}"(ret) : "{rax}"(n), "{rdi}"(a1), "{rsi}"(a2) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall3(n: int, a1: int, a2: int, a3: int) -> int {

    let mut ret : int = 0;

    asm!("syscall" : "={rax}"(ret) : "{rax}"(n), "{rdi}"(a1), "{rsi}"(a2), "{rdx}"(a3) : "rcx", "r11", "memory" : "volatile");

    return ret;
}

#[inline(always)]
pub unsafe fn syscall6(n: int,
                       a1: int, a2: int, a3: int,
                       a4: int, a5: int, a6: int) -> int {
    let mut ret: int = 0;

    asm!("syscall": "={rax}"(ret) :
         "{rax}"(n),
         "{rdi}"(a1), "{rsi}"(a2), "{rdx}"(a3), "{r10}"(a4), "{r8}"(a5), "{r9}"(a6) :
         "{rcx}", "r11", "memory" : "volatile");

    ret
}
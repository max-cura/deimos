core::arch::global_asm!(
    r#"
        .section ".text.start"
        .globl _start
        _start:
        .option push
        .option norelax
            la gp, __global_pointer$
        .option pop
            j {start_staged}
"#,
    start_staged = sym start_staged
);

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn start_staged() -> ! {
    unsafe extern "C" {
        static __bss_start: [u64; 0];
        static __bss_end: [u64; 0];
        static __stack_init: [u64; 0];
    }
    core::arch::naked_asm!(
        "csrw mie, zero",
        "li t1, {MXSTATUS_MAEE} | {MXSTATUS_THEADISAEE}
         csrs {MXSTATUS_CSR}, t1",
        "li t2, {MCOR_BHT}|{MCOR_BTB}|{MCOR_INV}|{MCOR_BOTH}
         csrw {MCOR_CSR}, t2",
        "la t0, {BSS_START}
             la t1, {BSS_END}
        1:   bgeu t0, t1, 2f
             sd zero, 0(t0)
             addi t0, t0, 8
             j 1b
        2:   ",
        "la sp, {STACK_INIT}
             andi sp, sp, -16
             add fp, sp, zero
             j {KERNEL_START}",
        MXSTATUS_MAEE = const 1 << 21,
        MXSTATUS_THEADISAEE = const 1 << 22,
        // mxstatus is SoC-specific, so we can't reference it by name
        MXSTATUS_CSR = const 0x7c0,
        MCOR_BHT = const 1 << 16,
        MCOR_BTB = const 1 << 17,
        MCOR_INV = const 1 << 4,
        MCOR_BOTH = const 3,
        // mcor is also SoC-specific
        MCOR_CSR = const 0x7c2,

        BSS_START = sym __bss_start,
        BSS_END = sym __bss_end,
        STACK_INIT = sym __stack_init,
        KERNEL_START = sym crate::kernel_start,
    )
}

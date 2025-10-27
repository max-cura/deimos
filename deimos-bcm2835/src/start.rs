unsafe extern "C" {
    static __bss_start: [u32; 0];
    static __bss_end: [u32; 0];
    static __stack_init: [u32; 0];
}

core::arch::global_asm!(r#"
.section ".text.start"
.globl _start
_start:
    mrs r0, cpsr
    and r0, r0, {CLEAR_MODE_MASK}
    orr r0, r0, {SUPER_MODE}
    orr r0, r0, {CLEAR_MODE_IRQ_FIQ}
    msr cpsr, r0
    mov r0, #0
    mcr p15, 0, r0, c7, c5, 4
    mov r0, #0
    ldr r1, ={BSS_START}
    ldr r2, ={BSS_END}
    subs r2, r2, r1
    bcc 3f
2:
    strb r0, [r1], #1
    subs r2, r2, #1
    bne 2b
3:

    ldr sp, ={STACK_INIT}
    add sp, sp, #0x20000
    mov r0, sp
    mov fp, #0
    bl {KERNEL_START}
"#,
    CLEAR_MODE_MASK = const !0b11111u32,
    SUPER_MODE = const 0b10011u32,
    CLEAR_MODE_IRQ_FIQ = const (1u32 << 7) | (1u32 << 6),
    BSS_START = sym __bss_start,
    BSS_END = sym __bss_end,
    STACK_INIT = sym __stack_init,
    KERNEL_START = sym crate::__kernel_start,
);


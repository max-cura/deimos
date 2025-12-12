use core::arch::{global_asm, naked_asm};

global_asm!(
    r#"
        .section ".text.start"
        .globl _start
    _start:
        b start_stage0
    "#
);

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn start_stage0() -> ! {
    unsafe extern "C" {
        static __bss_start: [u64; 0];
        static __bss_end: [u64; 0];
        static __stack_end: [u64; 0];
    }
    #[allow(unused_unsafe)]
    unsafe {
        naked_asm!(r#"
                @ Force supervisor mode and disable interrupts
                cpsid aif, #0b10011

                @ Enable NEON/VFP
                mrc p15, 0, r0, c1, c0, 2
                orr r0, r0, #0x00f00000
                mcr p15, 0, r0, c1, c0, 2
                isb
                mov r0, #0x40000000
                vmsr fpexc, r0

                @ MPIDR (multiprocessor affinity). Lowest four bits are CPU ID
                mrc p15, 0, r0, c0, c0, 5
                ands r0, r0, #3
                beq 3f
            2:
                b 2b
            3:
                mov r0, #0
                ldr r1, ={BSS_START}
                ldr r2, ={BSS_END}
                subs r2, r2, r1
                bls 5f
            4:
                strb r0, [r1], #1
                subs r2, r2, #1
                bne 4b
            5:
                ldr sp, ={STACK_INIT}
                mov fp, #0
                b {KERNEL_MAIN}
                "#,
            BSS_START = sym __bss_start,
            BSS_END = sym __bss_end,
            STACK_INIT = sym __stack_end,
            KERNEL_MAIN = sym start_stage1,
        )
    }
}

pub extern "C" fn start_stage1() -> ! {
    crate::kernel_start();

    reboot();
}

pub fn reboot() -> ! {
    loop {}
}

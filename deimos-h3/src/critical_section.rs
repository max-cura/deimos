use core::arch::asm;

struct H3CriticalSection;

critical_section::set_impl!(H3CriticalSection);

const CPSR_AIF_MASK: u32 = 0x1c0;

unsafe impl critical_section::Impl for H3CriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        let orig_mask: u32;
        unsafe {
            asm!(r#"
                mrs {t}, cpsr
                orr {u}, {t}, {AIF_MASK}
                msr cpsr, {t}
                "#,
                t = out(reg) orig_mask,
                u = out(reg) _,
                AIF_MASK = const CPSR_AIF_MASK,
            );
        }
        orig_mask & CPSR_AIF_MASK
    }

    unsafe fn release(restore_state: critical_section::RawRestoreState) {
        debug_assert_eq!(restore_state & !CPSR_AIF_MASK, 0, "invalid restore state");
        unsafe {
            asm!(r#"
                mrs {t}, cpsr
                orr {t}, {t}, {u}
                msr cpsr, {t}
                "#,
                t = out(reg) _,
                u = in(reg) (restore_state & CPSR_AIF_MASK),
            );
        }
    }
}

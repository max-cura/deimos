use critical_section::RawRestoreState;

struct CriticalSection;
critical_section::set_impl!(CriticalSection);
const CPSR_IF_MASK: u32 = 0x0c0;
unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        let orig_mask: u32;
        unsafe {
            core::arch::asm!(r#"
            mrs {t}, cpsr
            orr {u}, {t}, {IF_MASK}
            msr cpsr, {u}
        "#,
                t = out(reg) orig_mask,
                u = out(reg) _,
                IF_MASK = const CPSR_IF_MASK,
            );
        }
        orig_mask
    }

    unsafe fn release(restore_state: RawRestoreState) {
        // We only want to restore the state of the I and F bits in the CPSR. If we touch anything
        // else, then the behaviour of the program is undefined since we don't know what state the
        // processor is in.
        let restore_state = restore_state & CPSR_IF_MASK;
        const CPSR_CLEAR_IF_MASK: u32 = !CPSR_IF_MASK;
        unsafe {
            core::arch::asm!(r#"
            mrs {t}, cpsr
            and {t}, {t}, {CLR_IF_MASK}
            orr {t}, {t}, {restore}
            msr cpsr, {t}
        "#,
                t = out(reg) _,
                restore = in(reg) restore_state,
                CLR_IF_MASK = const CPSR_CLEAR_IF_MASK
            )
        }
    }
}

use critical_section::RawRestoreState;

pub struct SingleHartCriticalSection;
critical_section::set_impl!(SingleHartCriticalSection);

unsafe impl critical_section::Impl for SingleHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        let mut mstatus: usize;
        unsafe { core::arch::asm!(r#"csrrci {}, mstatus, 0b1000"#, out(reg) mstatus) }
        (mstatus & 0xffff_ffff) as u32
    }

    unsafe fn release(restore_state: RawRestoreState) {
        let was_active = (restore_state & 0x8) != 0;
        if was_active {
            unsafe {
                core::arch::asm!(r#"csrsi mstatus, 0b1000"#);
            }
        }
    }
}

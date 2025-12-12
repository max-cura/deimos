use crate::{arch::dsb, define_coprocessor_registers};

define_coprocessor_registers! {
    [safe write] dccimvac => p15 0 c7 c14 1;
    [safe write] dcimvac => p15 0 c7 c6 1;
    [safe read] ccsidr => p15 1 c0 c0 0;
}

#[allow(unused)]
pub fn cache_line_size() -> usize {
    return 1usize << ((ccsidr::read_raw() & 0x3) + 2 /* word count */ + 2/* words to bytes */);
}

#[allow(unused)]
pub fn dcache_clean_invalidate_range(start: usize, end: usize) {
    let line_size = cache_line_size();
    let mut mva = start & !(line_size - 1);
    while mva < end {
        dccimvac::write_raw(mva as u32);
        mva = mva + line_size;
    }
    dsb();
}

#[allow(unused)]
pub fn dcache_invalidate_range(start: usize, end: usize) {
    let line_size = cache_line_size();
    let mut mva = start & !(line_size - 1);
    while mva < end {
        dcimvac::write_raw(mva as u32);
        mva += line_size;
    }
    dsb();
}

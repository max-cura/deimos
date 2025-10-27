const CHANNEL_0_14_OFFSET_FROM_PERI_BASE: usize = 0x00_7000;
const CHANNEL_15_OFFSET_FROM_PERI_BASE: usize = 0xE0_5000;
const CHANNEL_STRIDE: usize = 0x100;
const PERI_BASE_ARM: *mut u32 = core::ptr::with_exposed_provenance_mut(0x2000_0000);

pub fn channel_ptr(channel: usize) -> *mut u32 {
    let channel_offset = if channel <= 14 {
        CHANNEL_0_14_OFFSET_FROM_PERI_BASE + channel * 0x100
    } else if channel == 15 {
        CHANNEL_15_OFFSET_FROM_PERI_BASE
    } else {
        panic!("no channels >15");
    };
    unsafe { PERI_BASE_ARM.byte_add(channel_offset) }
}

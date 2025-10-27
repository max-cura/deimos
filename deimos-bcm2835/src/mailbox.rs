use crate::println;
use core::alloc::Layout;
use core::arch::asm;

pub mod dma_channels;
pub mod revision;

pub fn dump_configuration() {
    println!("Dumping board configuration:");
    let rev = revision::query();
    println!("Board revision: {rev}");
    let mut mac = [0u8; 6];
    send_message(0x0001_0003, &mut mac);
    println!("MAC address={:X?}", mac);
    let mut arm_mem = [0u32; 2];
    send_message(0x0001_0005, bytemuck::cast_slice_mut(&mut arm_mem));
    println!("ARM memory base address: {:08x}", arm_mem[0]);
    println!("ARM memory size: {:08x}", arm_mem[1]);
    let mut vc_mem = [0u32; 2];
    send_message(0x0001_0006, bytemuck::cast_slice_mut(&mut vc_mem));
    println!("VC memory base address: {:08x}", vc_mem[0]);
    println!("VC memory size: {:08x}", vc_mem[1]);
}

// 16-byte aligned buffer
// channel 8 for ARM -> VC
// buffer format:
//  0 => buffer size in bytes including header, end tag, and padding
//  4 => code
//      request:  0000_0000 => process request
//      response: 8000_0000 => request successful
//                8000_0001 => error parsing request buffer
//  8 => (sequence of concatenated tags)
//  ? => 0000_0000 (end tag)
//  ? => padding...
// wiki doesn't actually say what the padding is so I'm going to assume that I should pad it to 16B
//
// tag format:
//  0 => tag identifier
//  4 => value buffer size (bytes)
//  8 => request code
//      request: 0000_0000 => request
//      response: 8000_0000 => response
//                | 7fff_ffff => value length in bytes
//  C => (value buffer)
//  ? => pad to 4 bytes

#[repr(C)]
struct MessageHeader {
    size: u32,
    code: u32,
}
#[repr(C)]
struct TagHeader {
    id: u32,
    value_size: u32,
    code: u32,
}

pub fn send_message(tag_id: u32, value: &mut [u8]) {
    let layout = Layout::from_size_align(
        (size_of::<MessageHeader>() + size_of::<TagHeader>() + ((value.len() + 3) & !3) + 4 + 0)
            & !0,
        16,
    )
    .expect("layout is valid");
    let message: *mut u32 = unsafe { alloc::alloc::alloc_zeroed(layout) }.cast();
    // println!("Allocated message buffer");
    assert!(!message.is_null());
    unsafe {
        message
            .offset(0)
            .cast::<MessageHeader>()
            .write_volatile(MessageHeader {
                size: layout.size() as u32,
                code: 0,
            });
        message
            .offset(2)
            .cast::<TagHeader>()
            .write_volatile(TagHeader {
                id: tag_id,
                value_size: value.len() as u32,
                code: 0,
            });
    }
    // println!("Wrote message initials");
    if !send_message_raw(message, layout.size()) {
        panic!("mailbox message failure");
    }
    // println!("Received return message");
    for i in 0..value.len() {
        let x = unsafe { message.offset(5).cast::<u8>().byte_add(i) };
        // println!("v[i] <- {x:?}");
        let y = unsafe { x.read_volatile() };
        value[i] = y;
    }
    // println!("Copied response to buffer");
    unsafe { alloc::alloc::dealloc(message.cast(), layout) };
    // println!("Deallocated message");
}

// There are two mailboxes:
//  0 is VC -> ARM
//  1 is ARM -> VC
// ARM should never read MB 1 or write MB 0

// On 2835, the ARM has no L2, so the ARM CPU is made to use the GPU L2 cache, which basically ends
// up meaning that the VideoCore MMU maps the ARM's view of memory to the 0x4 alias.
const BUS_ALIAS: usize = 0x4000_0000;

const TAGS_CHANNEL: u32 = 0x0000_0008;
const CHANNEL_MASK: u32 = 0x0000_000f;

fn send_message_raw(message: *mut u32, len: usize) -> bool {
    assert!(len >= 2);
    assert!(message.is_aligned_to(16));

    // We need to:
    //  1. ensure that any existing bus transactions related to `message` have completed
    //  2. ensure that L1 cache is flushed, since the VC can't see the ARM L1 cache
    //  3. ensure that the compiler knows that mechanisms beyond its purview may mutate `message`
    const BASE: usize = 0x2000_b880;
    unsafe {
        asm!(r#"
            mcr p15, 0, {z}, c7, c10, 4 // dsb
            mcr p15, 0, {z}, c7, c14, 0 // clean and invalidate entire dcache
        2:
            ldr {t0}, [{base}, #{STATUS1}]
            movs {t0}, {t0}
            bmi 2b

            orr {tmsg}, {msg}, {bus_alias}
            orr {tmsg}, {tmsg}, #{TAGS_CHANNEL}
            str {tmsg}, [{base}, #{WRITE1}]
        3:
            ldr {t0}, [{base}, #{STATUS0}]
            movs {t0}, {t0}, lsl #1
            bmi 3b

            ldr {t0}, [{base}, #{READ0}]
            and {t0}, {t0}, #{CHANNEL_MASK}
            cmp {t0}, #8
            bne 3b

            mcr p15, 0, {z}, c7, c10, 4 // dsb

            "#,
            z = in(reg) 0u32,
            t0 = out(reg) _,
            msg = in(reg) message,
            tmsg = out(reg) _,
            base = in(reg) BASE,
            READ0 = const 0x00,
            STATUS0 = const 0x18,
            WRITE1 = const 0x20,
            STATUS1 = const 0x38,
            bus_alias = const BUS_ALIAS,
            CHANNEL_MASK = const CHANNEL_MASK,
            TAGS_CHANNEL = const TAGS_CHANNEL,
        )
    }

    unsafe { message.offset(1).read_volatile() == 0x8000_0000u32 }
}

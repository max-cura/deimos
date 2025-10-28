use core::alloc::Layout;

use alloc::vec::Vec;
use sulfur::dilf::{Dst, Len, Loader, Nxt, Op, Src};

use crate::{
    dma::{Executive, Timing},
    print, println,
};

fn layout<T>(n: usize) -> Layout {
    let (layout, stride) = Layout::new::<T>().repeat(n).unwrap();
    assert_eq!(stride, size_of::<T>());
    layout
}

fn test_rt_from_length(sizes: &[usize], count: usize, channel: usize) {
    fn test(size: usize, count: usize, channel: usize) -> Vec<Timing> {
        let mut executive = Executive::new(0, 1, 2, 0);

        let dst = executive.load_chunk(Some("dst"), 0, layout::<u8>(size), None);
        let src = executive.load_chunk(Some("src"), 0, layout::<u8>(size), None);
        executive.load_ops([Op {
            flags: 0x5400,
            dst: Dst::data_ref(0, 0),
            src: Src::data_ref(1, 0),
            len: Len::fixed(size),
            nxt: Nxt::end(),
        }]);
        executive.map_routine("main", 0);

        let mut timings = Vec::new();

        for _ in 0..count {
            for i in 0..size {
                unsafe { dst.byte_add(i).write_volatile(0) };
                unsafe { src.byte_add(i).write_volatile((i & 0xff) as u8) };
            }
            timings.push(executive.execute("main", channel));
        }

        timings
    }

    println!();
    println!("Transfer Length / Runtime Correlation");
    for (i, &size) in sizes.iter().enumerate() {
        if i.is_multiple_of(2) {
            print!("\x1b[42m");
        }
        let timings = test(size, count, channel);
        print!("\t{size}");
        for timing in timings {
            print!("\t{}", timing.cycles());
        }
        println!("\x1b[0m");
    }
}

fn test_rt_unaligned(count: usize, channel: usize) {
    fn test(
        dst_align_offset: usize,
        src_align_offset: usize,
        len: usize,
        count: usize,
        channel: usize,
    ) -> Vec<Timing> {
        let mut executive = Executive::new(0, 128, 2, 0);
        let _dst = executive.load_chunk(Some("dst"), 0, layout::<u32>(4), None);
        let _src = executive.load_chunk(Some("src"), 0, layout::<u32>(4), None);
        for i in 0..128 {
            executive.load_ops([Op {
                flags: if i == 127 { 0x5400 } else { 0x6400 },
                dst: Dst::data_ref(0, dst_align_offset),
                src: Src::data_ref(1, src_align_offset),
                len: Len::fixed(len),
                nxt: if i == 127 {
                    Nxt::end()
                } else {
                    Nxt::op_ref(i + 1)
                },
            }]);
        }
        executive.map_routine("main", 0);
        let mut timings = Vec::new();
        for _ in 0..count {
            timings.push(executive.execute("main", channel));
        }
        timings
    }
    println!();
    println!("Unaligned-Dst Transfers (4B)");
    for align in [0usize, 1, 2, 3] {
        if align.is_multiple_of(2) {
            print!("\x1b[42m");
        }
        let timings = test(align, 0, 4, count, channel);
        print!("\t{align}");
        for timing in timings {
            print!("\t{}", timing.cycles());
        }
        println!("\x1b[0m");
    }
    println!();
    println!("Unaligned-Dst Transfers (1B)");
    for align in [0usize, 1, 2, 3] {
        if align.is_multiple_of(2) {
            print!("\x1b[42m");
        }
        let timings = test(align, 0, 1, count, channel);
        print!("\t{align}");
        for timing in timings {
            print!("\t{}", timing.cycles());
        }
        println!("\x1b[0m");
    }
    println!();
    println!("2-Word Transfer");
    print!("\x1b[42m");
    let timings = test(0, 0, 8, count, channel);
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
}

fn test_rt_caching_behaviour(count: usize, channel: usize) {
    fn test_all_different(count: usize, channel: usize) -> Vec<Timing> {
        let mut executive = Executive::new(0, 128, 2 * 128, 0);
        for _ in 0..256 {
            let _ = executive.load_chunk(None, 0, layout::<u128>(16), None);
        }
        for i in 0..128 {
            executive.load_ops([Op {
                flags: if i == 127 { 0x5400 } else { 0x6400 },
                dst: Dst::data_ref(i * 2, 0),
                src: Src::data_ref(i * 2 + 1, 0),
                len: Len::fixed(16 * 16),
                nxt: if i == 127 {
                    Nxt::end()
                } else {
                    Nxt::op_ref(i + 1)
                },
            }]);
        }
        executive.map_routine("main", 0);
        let mut timings = Vec::new();
        for _ in 0..count {
            timings.push(executive.execute("main", channel));
        }
        timings
    }
    fn test_all_same(count: usize, channel: usize) -> Vec<Timing> {
        let mut executive = Executive::new(0, 128, 2 * 128, 0);
        let _dst = executive.load_chunk(Some("dst"), 0, layout::<u128>(16), None);
        let _src = executive.load_chunk(Some("src"), 0, layout::<u128>(16), None);
        for i in 0..128 {
            executive.load_ops([Op {
                flags: if i == 127 { 0x5400 } else { 0x6400 },
                dst: Dst::data_ref(0, 0),
                src: Src::data_ref(1, 0),
                len: Len::fixed(16 * 16),
                nxt: if i == 127 {
                    Nxt::end()
                } else {
                    Nxt::op_ref(i + 1)
                },
            }]);
        }
        executive.map_routine("main", 0);
        let mut timings = Vec::new();
        for _ in 0..count {
            timings.push(executive.execute("main", channel));
        }
        timings
    }
    println!();
    println!("All-different transfer (256B)");
    let timings = test_all_different(count, channel);
    print!("\x1b[42m");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
    println!();
    println!("All-same transfer (256B)");
    let timings = test_all_same(count, channel);
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!();
}

pub fn all(channel: usize) {
    test_rt_from_length(
        &[
            1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
        ],
        25,
        channel,
    );
    test_rt_unaligned(25, channel);
    test_rt_caching_behaviour(25, channel);
}

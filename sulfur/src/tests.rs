use core::alloc::Layout;

use crate::{
    Timing,
    dilf::{Dst, Executor, Frame, Len, Nxt, Op, Src},
};
use alloc::vec::Vec;

use crate::{print, println};

fn layout<T>(n: usize) -> Layout {
    let (layout, stride) = Layout::new::<T>().repeat(n).unwrap();
    assert_eq!(stride, size_of::<T>());
    layout
}

fn test_rt_from_length<E: Executor>(executor: &mut E, sizes: &[usize], count: usize) {
    fn test<E: Executor>(executor: &mut E, size: usize, count: usize) -> Vec<Timing> {
        let mut frame = E::Frame::new(1, 0);

        let dst = frame.load_chunk(Some("dst"), 0, layout::<u8>(size), None);
        let src = frame.load_chunk(Some("src"), 0, layout::<u8>(size), None);
        frame.load_ops([Op {
            flags: 0x5400,
            dst: Dst::data_ref(0, 0),
            src: Src::data_ref(1, 0),
            len: Len::fixed(size),
            nxt: Nxt::end(),
        }]);
        frame.map_routine("main", 0);

        let mut timings = Vec::new();

        for _ in 0..count {
            for i in 0..size {
                unsafe { dst.byte_add(i).write_volatile(0) };
                unsafe { src.byte_add(i).write_volatile((i & 0xff) as u8) };
            }
            timings.push(executor.execute(&mut frame, "main"));
        }

        timings
    }

    println!();
    println!("Transfer Length / Runtime Correlation");
    for (i, &size) in sizes.iter().enumerate() {
        if i.is_multiple_of(2) {
            print!("\x1b[42m");
        }
        let timings = test(executor, size, count);
        print!("\t{size}");
        for timing in timings {
            print!("\t{}", timing.cycles());
        }
        println!("\x1b[0m");
    }
}

fn test_rt_unaligned<E: Executor>(executor: &mut E, count: usize) {
    fn test<E: Executor>(
        executor: &mut E,
        dst_align_offset: usize,
        src_align_offset: usize,
        len: usize,
        count: usize,
    ) -> Vec<Timing> {
        let mut frame = E::Frame::new(128, 0);
        let _dst = frame.load_chunk(Some("dst"), 0, layout::<u32>(4), None);
        let _src = frame.load_chunk(Some("src"), 0, layout::<u32>(4), None);
        for i in 0..128 {
            frame.load_ops([Op {
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
        frame.map_routine("main", 0);
        let mut timings = Vec::new();
        for _ in 0..count {
            timings.push(executor.execute(&mut frame, "main"));
        }
        timings
    }

    println!();
    println!("Unaligned-Dst Transfers (4B)");
    for align in [0usize, 1, 2, 3] {
        if align.is_multiple_of(2) {
            print!("\x1b[42m");
        }
        let timings = test(executor, align, 0, 4, count);
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
        let timings = test(executor, align, 0, 1, count);
        print!("\t{align}");
        for timing in timings {
            print!("\t{}", timing.cycles());
        }
        println!("\x1b[0m");
    }

    println!();
    println!("2-Word Transfer");
    print!("\x1b[42m");
    let timings = test(executor, 0, 0, 8, count);
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
}

fn test_rt_caching_behaviour<E: Executor>(executor: &mut E, count: usize) {
    fn test_all_different<E: Executor>(executor: &mut E, count: usize) -> Vec<Timing> {
        let mut frame = E::Frame::new(128, 0);
        for _ in 0..256 {
            let _ = frame.load_chunk(None, 0, layout::<u128>(16), None);
        }
        for i in 0..128 {
            frame.load_ops([Op {
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
        frame.map_routine("main", 0);
        let mut timings = Vec::new();
        for _ in 0..count {
            timings.push(executor.execute(&mut frame, "main"));
        }
        timings
    }
    fn test_all_same<E: Executor>(executor: &mut E, count: usize) -> Vec<Timing> {
        let mut frame = E::Frame::new(128, 0);
        let _dst = frame.load_chunk(Some("dst"), 0, layout::<u128>(16), None);
        let _src = frame.load_chunk(Some("src"), 0, layout::<u128>(16), None);
        for i in 0..128 {
            frame.load_ops([Op {
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
        frame.map_routine("main", 0);
        let mut timings = Vec::new();
        for _ in 0..count {
            timings.push(executor.execute(&mut frame, "main"));
        }
        timings
    }

    println!();
    println!("All-different transfer (256B)");
    let timings = test_all_different(executor, count);
    print!("\x1b[42m");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");

    println!();
    println!("All-same transfer (256B)");
    let timings = test_all_same(executor, count);
    print!("\x1b[42m");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
}

pub fn all<E: Executor>(executor: &mut E, count: usize) {
    test_rt_from_length(
        executor,
        &[
            1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
        ],
        count,
    );
    test_rt_unaligned(executor, count);
    test_rt_caching_behaviour(executor, count);
}

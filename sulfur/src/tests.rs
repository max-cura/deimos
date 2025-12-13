use core::alloc::Layout;

use crate::{
    Timing,
    dilf::{Dst, Executor, Frame, Len, Nxt, Op, OpFieldId, Src},
};
use alloc::vec;
use alloc::vec::Vec;

use crate::{print, println};

pub static BG: &'static str = "\x1b[45m";

pub fn layout<T>(n: usize) -> Layout {
    let (layout, stride) = Layout::new::<T>().repeat(n).unwrap();
    assert_eq!(stride, size_of::<T>());
    layout
}
pub fn layout_sa(s: usize, a: usize) -> Layout {
    Layout::from_size_align(s, a).unwrap()
}
pub fn layout_sas(sa: usize) -> Layout {
    layout_sa(sa, sa)
}

fn test_xor_perf<E: Executor>(executor: &mut E, count: usize) {
    const REPLICATIONS: usize = 1024;
    const OPC: usize = 3;
    fn test<E: Executor>(executor: &mut E, count: usize) -> (Vec<Timing>, Vec<Timing>) {
        let mut frame = E::Frame::new(OPC * REPLICATIONS, 0);
        let lut = frame.load_chunk(Some("lut"), 0, layout_sas(0x1_0000), None);
        let lhs = frame.load_chunk(Some("lhs"), 0, layout::<u32>(REPLICATIONS / 4), None);
        let rhs = frame.load_chunk(Some("rhs"), 0, layout::<u32>(REPLICATIONS / 4), None);
        let out = frame.load_chunk(Some("out"), 0, layout::<u32>(REPLICATIONS / 4), None);
        for i in 0..256 {
            for j in 0..256 {
                let v = (i as u8) ^ (j as u8);
                unsafe { lut.add(i * 256 + j).write_volatile(v) };
            }
        }
        for i in 0..REPLICATIONS {
            unsafe {
                lhs.add(i).write_volatile(i as u8);
                rhs.add(i)
                    .write_volatile((REPLICATIONS + 67 + i * 0x12345678) as u8);
            }
        }

        for i in 0..REPLICATIONS {
            let j = i * OPC;
            frame.load_ops([
                Op {
                    flags: 0x6402,
                    dst: Dst::op_ref_field(j + 2, OpFieldId::Src, 0),
                    src: Src::data_ref(1, i),
                    len: Len::fixed(1),
                    nxt: Nxt::op_ref(j + 1),
                },
                Op {
                    flags: 0x6402,
                    dst: Dst::op_ref_field(j + 2, OpFieldId::Src, 1),
                    src: Src::data_ref(2, i),
                    len: Len::fixed(1),
                    nxt: Nxt::op_ref(j + 2),
                },
                Op {
                    flags: if i == REPLICATIONS - 1 {
                        0x5400
                    } else {
                        0x6400
                    },
                    dst: Dst::data_ref(3, i),
                    src: Src::data_ref(0, 0),
                    len: Len::fixed(1),
                    nxt: if i == REPLICATIONS - 1 {
                        Nxt::end()
                    } else {
                        Nxt::op_ref(j + 3)
                    },
                },
            ])
        }
        frame.map_routine("main", 0);

        let mut timings_dma = vec![];
        let mut timings_cpu = vec![];

        for _ in 0..count {
            timings_dma.push(executor.execute(&mut frame, "main"));
            for i in 0..REPLICATIONS {
                assert_eq!(unsafe { out.add(i).read_volatile() }, unsafe {
                    lhs.add(i).read_volatile() ^ rhs.add(i).read_volatile()
                });
            }
            // unsafe {
            //     println!(
            //         "out={:02x?} lhs={:02x?} rhs={:02x?} chk={:02x?} lut={:#?}",
            //         out.cast_array::<4>().read_volatile(),
            //         lhs.cast_array::<4>().read_volatile(),
            //         rhs.cast_array::<4>().read_volatile(),
            //         lut.as_ptr()
            //     )
            // };
            timings_cpu.push(executor.time(|| {
                for i in 0..REPLICATIONS / 4 {
                    unsafe {
                        out.cast::<u32>().add(i).write_volatile(
                            lhs.cast::<u32>().add(i).read_volatile()
                                ^ rhs.cast::<u32>().add(i).read_volatile(),
                        )
                    };
                }
            }));
        }

        (timings_dma, timings_cpu)
    }

    println!();
    println!("XOR8x{} performance", REPLICATIONS);
    let (timings_dma, timings_cpu) = test(executor, count);
    print!("{BG}DMA");
    for timing in timings_dma {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
    print!("CPU");
    for timing in timings_cpu {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
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
            print!("{BG}");
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
        let _dst = frame.load_chunk(Some("dst"), 0, layout_sas(256), None);
        let _src = frame.load_chunk(Some("src"), 0, layout_sas(256), None);
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
    for align in 0usize..64 {
        if align.is_multiple_of(2) {
            print!("{BG}");
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
    for align in 0usize..64 {
        if align.is_multiple_of(2) {
            print!("{BG}");
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
    print!("{BG}");
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
    fn test_src_same<E: Executor>(executor: &mut E, count: usize) -> Vec<Timing> {
        let mut frame = E::Frame::new(128, 0);
        for _ in 0..256 {
            let _ = frame.load_chunk(None, 0, layout::<u128>(16), None);
        }
        for i in 0..128 {
            frame.load_ops([Op {
                flags: if i == 127 { 0x5400 } else { 0x6400 },
                dst: Dst::data_ref(i * 2, 0),
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
    fn test_dst_same<E: Executor>(executor: &mut E, count: usize) -> Vec<Timing> {
        let mut frame = E::Frame::new(128, 0);
        for _ in 0..256 {
            let _ = frame.load_chunk(None, 0, layout::<u128>(16), None);
        }
        for i in 0..128 {
            frame.load_ops([Op {
                flags: if i == 127 { 0x5400 } else { 0x6400 },
                dst: Dst::data_ref(0, 0),
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

    println!();
    println!("DMA-to-DMA intrinsic contention");
    let timings = test_all_different(executor, count);
    print!("{BG}\t=NONE");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");

    // println!();
    // println!("All-same transfer (128x256B)");
    let timings = test_all_same(executor, count);
    print!("\t=ALL");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");

    // println!();
    // println!("Src-same transfer (128x256B)");
    let timings = test_src_same(executor, count);
    print!("{BG}\t=SRC");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");

    let timings = test_dst_same(executor, count);
    print!("\t=DST");
    for timing in timings {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
}

pub fn all<E: Executor>(executor: &mut E, count: usize) {
    // test_xor_perf(executor, count);
    // test_rt_from_length(
    //     executor,
    //     &[
    //         1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
    //     ],
    //     count,
    // );
    // test_rt_unaligned(executor, count);
    // test_rt_caching_behaviour(executor, count);
}

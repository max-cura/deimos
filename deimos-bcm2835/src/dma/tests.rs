use core::alloc::Layout;

use alloc::vec;
use alloc::vec::Vec;
use sulfur::{
    Timing,
    dilf::{Dst, Executor, Frame, Len, Nxt, Op, Src},
    print, println,
    tests::{BG, layout_sas},
};

use crate::dma::Channel;

pub fn test_rt_l2<E: Executor>(executor: &mut E, count: usize) {
    fn test<E: Executor>(executor: &mut E, j: usize, count: usize) -> Vec<Timing> {
        let mut frame = E::Frame::new(128, 0);
        let _dst = frame.load_chunk(Some("dst"), 0, Layout::new::<u32>(), None);
        let dead_area = 0x0800_0000 + 0x10_000 * j;
        frame.load_ops((0..128).map(|i| Op {
            flags: if i == 127 { 0x5440 } else { 0x6440 },
            dst: Dst::data_ref(0, 0),
            src: Src::fixed_vc(dead_area + i * 0x100),
            len: Len::fixed(4),
            nxt: if i == 127 {
                Nxt::end()
            } else {
                Nxt::op_ref(i + 1)
            },
        }));
        frame.map_routine("main", 0);
        (0..count)
            .map(|_| executor.execute(&mut frame, "main"))
            .collect()
    }

    println!();
    println!("L2 cache comparison");
    let mut timings_hit: Vec<Timing> = vec![];
    let mut timings_miss: Vec<Timing> = vec![];
    for j in 0..count {
        let timings = test(executor, j, 2);
        timings_hit.push(timings[0]);
        timings_miss.push(timings[1]);
    }
    print!("\tMiss");
    for timing in timings_hit.iter() {
        print!("\t{}", timing.cycles());
    }
    println!();
    print!("\tHit");
    for timing in timings_miss.iter() {
        print!("\t{}", timing.cycles());
    }
    println!();
    print!("{BG}");
    print!("\tDiff");
    for (hit, miss) in timings_hit.iter().zip(timings_miss.iter()) {
        print!("\t{}", hit.cycles() - miss.cycles());
    }
    println!("\x1b[0m");
}

pub fn test_read_contention(executor: &mut Channel, count: usize) {
    const REPEAT: usize = 4096;
    fn test_contended_dst(
        executor: &mut Channel,
        count: usize,
    ) -> (Vec<Timing>, Vec<Timing>, Vec<Timing>) {
        let mut frame = <Channel as Executor>::Frame::new(REPEAT, 0);
        let dst = frame.load_chunk(Some("dst"), 0, layout_sas(4096), None);
        let _src = frame.load_chunk(Some("src"), 0, layout_sas(4096), None);

        frame.load_ops((0..REPEAT).map(|i| Op {
            flags: if i == REPEAT - 1 { 0x5400 } else { 0x6400 },
            dst: Dst::data_ref(0, 0),
            src: Src::data_ref(1, 0),
            len: Len::fixed(4),
            nxt: if i == REPEAT - 1 {
                Nxt::end()
            } else {
                Nxt::op_ref(i + 1)
            },
        }));
        frame.map_routine("main", 0);
        let contended = (0..count)
            .map(|_| executor.execute_contended_read(&mut frame, dst.as_ptr().cast(), "main"))
            .collect();
        let contended_unrelated = (0..count)
            .map(|_| executor.execute_contended_read(&mut frame, 0x0700_0000 as *mut u32, "main"))
            .collect();
        let uncontended = (0..count)
            .map(|_| executor.execute(&mut frame, "main"))
            .collect();
        (contended, contended_unrelated, uncontended)
    }

    println!();
    println!("CPU read / DMA write contention");
    let (timings_contended, timings_contended_unrelated, timings_uncontended) =
        test_contended_dst(executor, count);
    print!("{BG}\tCNTND");
    for timing in timings_contended.iter() {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
    print!("\tCNT NRL");
    for timing in timings_contended_unrelated.iter() {
        print!("\t{}", timing.cycles());
    }
    println!();
    print!("{BG}\tCLEAR");
    for timing in timings_uncontended.iter() {
        print!("\t{}", timing.cycles());
    }
    println!("\x1b[0m");
    print!("\tDiff");
    for (contended, uncontended) in timings_contended
        .iter()
        .zip(timings_contended_unrelated.iter())
    {
        print!(
            "\t{}",
            contended.cycles() as i64 - uncontended.cycles() as i64
        );
    }
    println!("\x1b[0m");
}

pub fn all(executor: &mut Channel, count: usize) {
    // test_rt_l2(executor, count);
    test_read_contention(executor, count);
}

use core::alloc::Layout;

use alloc::vec;
use alloc::vec::Vec;
use sulfur::{
    Timing,
    dilf::{Dst, Executor, Frame, Len, Nxt, Op, Src},
    print, println,
};

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
    print!("\x1b[42m");
    print!("\tDiff");
    for (hit, miss) in timings_hit.iter().zip(timings_miss.iter()) {
        print!("\t-{}", hit.cycles() - miss.cycles());
    }
    println!("\x1b[0m");
}

pub fn all<E: Executor>(executor: &mut E, count: usize) {
    test_rt_l2(executor, count);
}

use core::{alloc::Layout, arch::asm, cell::SyncUnsafeCell, ptr::NonNull, sync::atomic::Ordering};

use crate::{
    arch::dsb,
    dma::registers::{CS, TI},
    mailbox,
};
use alloc::{
    string::{String, ToString as _},
    vec::Vec,
};
use bcm2835_lpa::Peripherals;
use hashbrown::HashMap;
use sulfur::{
    Timing,
    dilf::{DataRef, Executor, Frame, Hole, Op, OpField, OpFieldId, OpFieldRef},
    println,
};
use tock_registers::LocalRegisterCopy;

mod raw;
mod registers;
mod tests;

core::arch::global_asm!(
    r#"
    .globl _interrupt_table
    .align 5
    _interrupt_table:
        b 100f // rest 00
        b 100f // undf 04
        b 100f // sysc 08
        b 100f // pabt 0c
        b 100f // dabt 10
        nop    //      14
        b 100f // irq  18
    200: // fiq
        // bl __xxx
        mrc p15, 0, r8, c15, c12, 1 // read cycle counter
        str r8, [r13] // write cycle counter to FIQ_STACK + 127
        ldr r8, =0x20007400
        ldr r9, [r8]
        orr r9, r9, #(1 << 2)
        str r9, [r8]
        mrs r9, cpsr // disable FIQ
        orr r9, r9, #(1 << 6)
        msr cpsr, r9
        sev // SEV
        subs pc, r14, #4 // return
    100: // nothing
        b 100b
    "#
);

#[unsafe(no_mangle)]
extern "C" fn __xxx() {
    println!("hi from xxx");
    loop {}
}

fn fiq_enable() {
    unsafe {
        asm!(
            r#"
                mrs {t0}, cpsr
                and {t0}, {t0}, #(~(1 << 6))
                msr cpsr, {t0}
            "#,
            t0 = out(reg) _
        )
    }
}

static FIQ_STACK: SyncUnsafeCell<[u32; 128]> = SyncUnsafeCell::new([0; 128]);

pub fn run_all(_peri: &Peripherals) {
    unsafe {
        asm!(
            r#"
                mrc p15, 0, {t0}, c15, c12, 0
                orr {t0}, {t0}, #1
                mcr p15, 0, {t0}, c15, c12, 0
                mcr p15, 0, {z}, c15, c12, 1
            "#,
            t0 = out(reg) _,
            z = in(reg) 0,
        );
    }

    let channels = mailbox::dma_channels::query();
    println!("Available DMA channels: {}", channels);
    let channel_no = channels
        .iter()
        .find(|c| *c > 3)
        .expect("at least one DMA channel should be available");
    println!("Selected DMA channel: {}", channel_no);

    let fiq_no = if channel_no <= 10 {
        16 + channel_no
    } else {
        27
    };

    // set up vector
    unsafe {
        asm!(
            r#"
            .extern _interrupt_table
                ldr {t}, =_interrupt_table
                mcr p15, 0, {t}, c12, c0, 0
                // prefetch flush
                mcr p15, 0, {z}, c7, c5, 4
            "#,
            z = in(reg) 0,
            t = out(reg) _,
        );
    }

    // set up FIQ buffer
    unsafe {
        asm!(
            r#"
                mrs {t0}, cpsr
                and {t0}, {t0}, #(~0b11111)
                orr {t0}, {t0}, #(0b10001)
                msr cpsr, {t0}

                mov r13, {t2}

                mrs {t0}, cpsr
                and {t0}, {t0}, #(~0b11111)
                orr {t0}, {t0}, #(0b10011)
                msr cpsr, {t0}

                // cpsid f, #0b10001
                // mov r13, {t2}
                // msr cpsr, {t0}
            "#,
            t0 = out(reg) _,
            t2 = in(reg) FIQ_STACK.get().add(127).addr()
        )
    }

    // set up FIQ
    unsafe {
        asm!(
            r#"
                ldr {t}, =0x2000b000
                str {fiq_ctrl}, [{t}, #0x20c]
                mcr p15, 0, {z}, c7, c10, 4
            "#,
            t = out(reg) _,
            z = in(reg) 0,
            fiq_ctrl = in(reg) (fiq_no | 0x80),
        );
    }

    println!("Set up FIQ machinery");

    let mut channel = Channel { channel_no };

    sulfur::tests::all(&mut channel, 25);
    tests::all(&mut channel, 25)

    // let mut executives = vec![];

    // for _ in 0..5 {
    //     let mut executive = Executive::new(0x10_000, 16, 16, 4);
    //     executive.load_chunk(Some("dst"), 0, Layout::new::<u32>(), Some(&[0; 4]));
    //     executive.load_chunk(
    //         Some("src"),
    //         0,
    //         Layout::new::<u32>(),
    //         Some(&u32::to_ne_bytes(0xdeadbeef)),
    //     );
    //     executive.load_ops([Op {
    //         flags: 0x5400,
    //         dst: Dst {
    //             data_ref: DataRef {
    //                 chunk: 0,
    //                 offset: 0,
    //             },
    //         },
    //         src: Src {
    //             data_ref: DataRef {
    //                 chunk: 1,
    //                 offset: 0,
    //             },
    //         },
    //         len: Len { fixed: 4 },
    //         nxt: Nxt { hole: Hole::End },
    //     }]);
    //     executive.map_routine("main", 0);
    //     executives.push(executive);
    //     // let timing = executive.execute("main", channel);
    //     // println!(
    //     //     // "DMA routine `main` executed in {:?} / {}cy",
    //     //     "1. {}",
    //     //     // timing.end - timing.begin,
    //     //     timing.cycle_end - timing.cycle_begin
    //     // );
    // }
    // for _ in 0..1 {
    //     let mut executive = Executive::new(0x10_000, 16, 16, 4);
    //     println!("created executive");
    //     let dst = executive.load_chunk(Some("dst"), 0, Layout::new::<[u8; 0x8000]>(), None);
    //     println!("loaded chunk `dst` = {dst:?}");
    //     let src = executive.load_chunk(Some("src"), 0, Layout::new::<[u8; 0x8000]>(), None);
    //     println!("loaded chunk `src` = {src:?}");

    //     println!("exec.op_count={}", executive.op_count);

    //     unsafe { asm!("sev") };
    //     // breaks at 0x75dc??? anyway
    //     for i in 0..0x8000 {
    //         unsafe {
    //             dst.byte_add(i).write_volatile(0);
    //             src.byte_add(i).write_volatile((i % 256) as u8);
    //         }
    //     }
    //     unsafe { asm!("sev") };
    //     println!("initialized `dst` and `src`");
    //     executive.load_ops([Op {
    //         flags: 0x5400,
    //         dst: Dst {
    //             data_ref: DataRef {
    //                 chunk: 0,
    //                 offset: 0,
    //             },
    //         },
    //         src: Src {
    //             data_ref: DataRef {
    //                 chunk: 1,
    //                 offset: 0,
    //             },
    //         },
    //         len: Len { fixed: 0x8000 },
    //         nxt: Nxt { hole: Hole::End },
    //     }]);
    //     println!("loaded operations");
    //     executive.map_routine("main", 0);
    //     println!("mapped routine `main`");
    //     let timing = executive.execute("main", channel);
    //     println!(
    //         "DMA routine `main` executed in {}",
    //         timing.cycle_end.wrapping_sub(timing.cycle_begin)
    //     );
    //     for i in 0..0x8000 {
    //         assert_eq!(
    //             unsafe { dst.add(i).read_volatile() },
    //             unsafe { src.add(i).read_volatile() },
    //             "mismatch at {i}"
    //         );
    //     }
    //     println!("all locations okay");
    //     executives.push(executive);
    // }

    // for (i, mut executive) in executives.into_iter().enumerate() {
    //     let timing = executive.execute("main", channel);
    //     println!(
    //         "{i} - {}",
    //         timing.cycle_end.wrapping_sub(timing.cycle_begin)
    //     );
    // }
}

#[derive(Debug)]
#[repr(C, align(0x20))]
struct CB {
    // LocalRegisterCopy<T,R> is #[repr(transparent)] over T
    ti: LocalRegisterCopy<u32, registers::TI::Register>,
    source_ad: u32,
    dest_ad: u32,
    txfr_len: u32,
    stride: u32,
    nextconbk: u32,
    pad: [u32; 2],
}
const _: () = assert!(size_of::<CB>() == 0x20);

pub struct Channel {
    channel_no: usize,
}
impl Channel {
    fn execute_contended_read(
        &mut self,
        frame: &mut DeviceFrame,
        contend: *mut u32,
        routine: &str,
    ) -> Timing {
        let op_idx = *frame.routine_map.get(routine).expect("unknown routine");
        let op_ptr = frame.resolve_op_ref(op_idx as u32);
        let op_vc_addr = op_ptr.as_ptr().addr(); //self.ptr_to_vc(op_ptr.as_ptr().cast());
        let channel_base = raw::channel_ptr(self.channel_no);

        let cycle_end: u32;

        let mut cs_value: LocalRegisterCopy<u32, registers::CS::Register> =
            LocalRegisterCopy::new(0);
        #[rustfmt::skip]
        cs_value.write(
            CS::ACTIVE::SET
                + CS::WAIT_FOR_OUTSTANDING_WRITES::SET
                + CS::PRIORITY::SET
        );
        crate::arch::clean_and_invalidate_entire_dcache::write_raw(0);

        core::sync::atomic::compiler_fence(Ordering::SeqCst);
        dsb();

        unsafe {
            asm!(
                r#"
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                    mcr p15, 0, {z}, c7, c14, 0 // clean & invalidate entire dcache
                    ldr {t0}, =0x100f0002
                    str {t0}, [{channel_base}, #{CS_OFFSET}]
                    mov {t0}, #7
                    str {t0}, [{channel_base}, #{DEBUG_OFFSET}]
                    str {op_vc_addr}, [{channel_base}, #{CONBLK_AD_OFFSET}]
                    mcr p15, 0, {z}, c7, c4, 0 // clean & invalidate entire dcache
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                    ldr {t0}, =4f
                    mcr p15, 0, {t0}, c7, c13, 1
                    mcr p15, 0, {z}, c7, c10, 4
                .align 5
                4:
                    mcr p15, 0, {z}, c15, c12, 1
                    str {cs_value}, [{channel_base}, #{CS_OFFSET}]
                3:
                    mcr p15, 0, {z}, c7, c6, 0 // invalidate the cache
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                    ldr {t0}, [{channel_base}, #{CS_OFFSET}]
                    ldr {k}, [{contend}] // CONTENTION !!!!!!!!!!!!!!!!!!!!
                    tst {t0}, #1
                    bne 3b
                    mrc p15, 0, {cyc_end}, c15, c12, 1 // cycle count

                    orr {t0}, {t0}, #2
                    str {t0}, [{channel_base}, #{CS_OFFSET}]
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                "#,
                z = inout(reg) 0u32 => _,
                t0 = inout(reg) 0u32 => _,
                cyc_end= out(reg) cycle_end,
                k = out(reg) _,

                contend = in(reg) contend,

                channel_base = in(reg) channel_base,
                op_vc_addr = in(reg) op_vc_addr,
                cs_value = in(reg) cs_value.get(),
                CS_OFFSET = const 0x00,
                CONBLK_AD_OFFSET = const 0x04,
                DEBUG_OFFSET = const 0x20,
            );
        }

        dsb();
        unsafe { asm!("mcr p15, 0, {t0}, c7, c6, 0", t0 = in(reg) 0) } // invalidate entire dcache
        dsb();

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        Timing {
            cycle_begin: 0,
            cycle_end,
        }
    }
}

impl Executor for Channel {
    type Frame = DeviceFrame;

    fn time(&self, f: impl FnOnce()) -> Timing {
        let cycle_end: u32;
        unsafe { asm!("mcr p15, 0, {}, c15, c12, 1", in(reg) 0) }
        f();
        unsafe { asm!("mrc p15, 0, {}, c15, c12, 1", out(reg) cycle_end) }
        Timing {
            cycle_begin: 0,
            cycle_end,
        }
    }

    fn execute(&mut self, frame: &mut DeviceFrame, routine: &str) -> Timing {
        let op_idx = *frame.routine_map.get(routine).expect("unknown routine");
        let op_ptr = frame.resolve_op_ref(op_idx as u32);
        let op_vc_addr = op_ptr.as_ptr().addr(); //self.ptr_to_vc(op_ptr.as_ptr().cast());
        let channel_base = raw::channel_ptr(self.channel_no);

        // println!("channel_base={channel_base:?}");
        // println!("op_vc_addr={op_vc_addr:08x}");

        // let st_begin_hi: u32;
        // let st_begin_lo: u32;
        // let st_end_hi: u32;
        // let st_end_lo: u32;
        let cycle_begin: u32;
        let cycle_end: u32;

        let mut cs_value: LocalRegisterCopy<u32, registers::CS::Register> =
            LocalRegisterCopy::new(0);
        #[rustfmt::skip]
        cs_value.write(
            CS::ACTIVE::SET
                + CS::WAIT_FOR_OUTSTANDING_WRITES::SET
                + CS::PRIORITY::SET
        );
        // println!("cs_value = {:08x}", cs_value.get());

        crate::arch::clean_and_invalidate_entire_dcache::write_raw(0);
        // unsafe { asm!("mcr p15, 0, {z}, c7, c14, 0", z = in(reg) 0) }

        // crate::timing::delay_millis(&unsafe { bcm2835_lpa::Peripherals::steal() }.SYSTMR, 1000);

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        dsb();

        unsafe {
            asm!(
                r#"
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                    mcr p15, 0, {z}, c7, c14, 0 // clean and invalidate entire dcache
                    mov {t0}, #2
                    // ldr {t0}, =0x100f0002
                    str {t0}, [{channel_base}, #{CS_OFFSET}]
                    mov {t0}, #7
                    str {t0}, [{channel_base}, #{DEBUG_OFFSET}]
                    str {op_vc_addr}, [{channel_base}, #{CONBLK_AD_OFFSET}]
                    mcr p15, 0, {z}, c7, c14, 0 // clean and invalidate entire dcache

                    mcr p15, 0, {z}, c7, c10, 4 // dsb

                    str {z}, [{p}] // FIQ_STACK[127] <- 0
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                    // mrs {t0}, cpsr // enable FIQ
                    // and {t0}, {t0}, #(~(1 << 6))
                    // msr cpsr, {t0}
                    mov {cc_begin}, #0 // cc_begin <- 0

                    // prefetch instruction cache line
                    ldr {t0}, =4f
                    mcr p15, 0, {t0}, c7, c13, 1
                    mcr p15, 0, {z}, c7, c10, 4
                4:
                .align 5
                    mcr p15, 0, {z}, c15, c12, 1 // set cycle counter to 0
                    str {cs_value}, [{channel_base}, #{CS_OFFSET}] // start DMA
                3:
                    // wfe
                    // mcr p15, 0, {z}, c7, c10, 4 // dsb
                    // ldr {cc_end}, [{p}]
                    // movs {cc_end}, {cc_end}
                    // beq 3b
                    
                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                    // nop
                    // nop
                    ldr {t0}, [{channel_base}, #{CS_OFFSET}]
                    tst {t0}, #1
                    bne 3b // loop while active
                    mrc p15, 0, {cc_end}, c15, c12, 1 // read cycle counter

                    // no longer active, clear END bit
                    orr {t0}, {t0}, #2
                    str {t0}, [{channel_base}, #{CS_OFFSET}]

                    mcr p15, 0, {z}, c7, c10, 4 // dsb
                "#,
                z = inout(reg) 0u32 => _,
                t0 = inout(reg) 0u32 => _,

                // st_begin_hi = out(reg) st_begin_hi,
                // st_begin_lo = out(reg) st_begin_lo,
                // st_end_hi = out(reg) st_end_hi,
                // st_end_lo = out(reg) st_end_lo,
                cc_begin = out(reg) cycle_begin,
                cc_end = out(reg) cycle_end,

                channel_base = in(reg) channel_base,
                op_vc_addr = in(reg) op_vc_addr,
                cs_value = in(reg) cs_value.get(),
                p = in(reg) unsafe { FIQ_STACK.get().add(127).addr() },
                CS_OFFSET = const 0x00,
                CONBLK_AD_OFFSET = const 0x04,
                DEBUG_OFFSET = const 0x20,
            );
        }

        // crate::timing::delay_millis(&unsafe { bcm2835_lpa::Peripherals::steal() }.SYSTMR, 1000);

        dsb();

        unsafe {
            asm!("mcr p15, 0, {t0}, c7, c6, 0", t0 = in(reg) 0);
        }

        dsb();
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        // let begin = Instant::from_raw(((st_begin_hi as u64) << 32) | (st_begin_lo as u64));
        // let end = Instant::from_raw(((st_end_hi as u64) << 32) | (st_end_lo as u64));

        Timing {
            // begin,
            // end,
            cycle_begin,
            cycle_end,
        }
    }
}

pub struct DeviceFrame {
    // arena: bumpalo::Bump,
    chunk_map: Vec<(NonNull<u8>, Layout)>,
    symbol_map: HashMap<String, (NonNull<u8>, usize)>,
    routine_map: HashMap<String, usize>,
    op_count: usize,
    ops_loaded: usize,
    op_layout: Layout,
    op_arena: NonNull<CB>,
    void: NonNull<u8>,
    void_size: usize,
    void_layout: Layout,
}
impl Drop for DeviceFrame {
    fn drop(&mut self) {
        for (chunk, layout) in self.chunk_map.iter() {
            unsafe { alloc::alloc::dealloc(chunk.as_ptr().cast(), *layout) }
        }
        unsafe { alloc::alloc::dealloc(self.op_arena.as_ptr().cast(), self.op_layout) };
        unsafe { alloc::alloc::dealloc(self.void.as_ptr().cast(), self.void_layout) };
    }
}
impl DeviceFrame {
    fn op_field_offset(&self, op_field_id: OpFieldId) -> usize {
        match op_field_id {
            OpFieldId::Dst => 2,
            OpFieldId::Src => 1,
            OpFieldId::Len => 3,
            OpFieldId::Nxt => 5,
        }
    }

    fn arm_to_vc(&self, arm: u32) -> u32 {
        if arm < 0x2000_0000 {
            arm | 0x8000_0000
        } else if 0x2000_0000 <= arm && arm < 0x2100_0000 {
            (arm - 0x2000_0000) + 0x7e00_0000
        } else {
            panic!("No ARM to VC mapping for: {arm:08x}")
        }
    }

    fn ptr_to_vc(&self, ptr: *mut u8) -> u32 {
        self.arm_to_vc(ptr.expose_provenance() as u32)
    }

    fn resolve_data_ref(&self, data_ref: DataRef) -> NonNull<u8> {
        let (chunk_base, chunk_layout) = self
            .chunk_map
            .get(data_ref.chunk as usize)
            .expect("data_ref.chunk should be in-range");
        assert!((data_ref.offset as usize) < chunk_layout.size());
        unsafe { chunk_base.add(data_ref.offset as usize) }
    }

    fn resolve_op_field_ref(&self, op_field_ref: OpFieldRef) -> NonNull<u32> {
        assert!((op_field_ref.op as usize) < self.op_count);
        let cb_ptr = unsafe { self.op_arena.add(op_field_ref.op as usize) }.cast::<u32>();
        unsafe {
            cb_ptr
                .add(self.op_field_offset(op_field_ref.field_id))
                .byte_add(op_field_ref.offset as usize)
        }
    }

    fn resolve_op_ref(&self, op_ref: u32) -> NonNull<CB> {
        assert!((op_ref as usize) < self.op_count);
        unsafe { self.op_arena.add(op_ref as usize) }
    }

    fn alloc_indirection(&mut self) -> NonNull<u32> {
        let layout = Layout::new::<u32>();
        let nn = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(layout) }).expect("OOM");
        self.chunk_map.push((nn, layout));
        nn.cast()
    }

    fn allocate_op_field_ref_indirection(&mut self, op_field_ref: OpFieldRef) -> NonNull<u32> {
        let nn = self.resolve_op_field_ref(op_field_ref);
        let as_vc = self.ptr_to_vc(nn.as_ptr().cast());
        let ind_ptr = self.alloc_indirection();
        // println!("allocated OpFieldRef indirection for {op_field_ref:?} = {ind_ptr:?}");
        unsafe { ind_ptr.write_volatile(as_vc) };
        ind_ptr
    }

    fn allocate_data_ref_indirection(&mut self, data_ref: DataRef) -> NonNull<u32> {
        let nn = self.resolve_data_ref(data_ref);
        let as_vc = self.ptr_to_vc(nn.as_ptr());
        let ind_ptr = self.alloc_indirection();
        // println!("allocated DataRef indirection for {data_ref:?} = {ind_ptr:?}");
        unsafe { ind_ptr.write_volatile(as_vc) };
        ind_ptr
    }

    fn allocate_op_ref_indirection(&mut self, op_ref: u32) -> NonNull<u32> {
        let nn = self.resolve_op_ref(op_ref);
        let as_vc = self.ptr_to_vc(nn.as_ptr().cast());
        let ind_ptr = self.alloc_indirection();
        // println!("allocated OpRef indirection for {op_ref} = {ind_ptr:?}");
        unsafe { ind_ptr.write_volatile(as_vc) };
        ind_ptr
    }

    fn translate_op(&mut self, op: Op) -> CB {
        let dst = op.dst();
        let src = op.src();
        let len = op.len();
        let nxt = op.nxt();

        // println!("translate_op: src={src:?}");

        let dest_ad = match dst {
            OpField::DataRef(data_ref) => {
                let nn = self.resolve_data_ref(*data_ref);
                self.ptr_to_vc(nn.as_ptr())
            }
            OpField::OpFieldRef(op_field_ref) => {
                let nn = self.resolve_op_field_ref(*op_field_ref);
                self.ptr_to_vc(nn.as_ptr().cast())
            }
            OpField::Fixed(fixed) => self.arm_to_vc(*fixed),
            OpField::Hole(hole) => match hole {
                Hole::End => unreachable!(),
                Hole::Void => {
                    if let OpField::Fixed(len) = len {
                        assert!(
                            (*len as usize) < self.void_size,
                            "Src=!void, but transfer length is greater than void_size"
                        );
                        self.ptr_to_vc(self.void.as_ptr())
                    } else {
                        panic!("Src=!void requires fixed-length transfer")
                    }
                }
                Hole::Param | Hole::Nil => *hole as u32,
            },
            _ => unreachable!(),
        };
        let source_ad = match src {
            OpField::DataRef(data_ref) => {
                let nn = self.resolve_data_ref(*data_ref);
                self.ptr_to_vc(nn.as_ptr())
            }
            OpField::DataRefIndirect(data_ref) => {
                let nn = self.allocate_data_ref_indirection(*data_ref);
                self.ptr_to_vc(nn.as_ptr().cast())
            }
            OpField::OpFieldRef(op_field_ref) => {
                let nn = self.resolve_op_field_ref(*op_field_ref);
                self.ptr_to_vc(nn.as_ptr().cast())
            }
            OpField::OpFieldRefIndirect(op_field_ref) => {
                let nn = self.allocate_op_field_ref_indirection(*op_field_ref);
                self.ptr_to_vc(nn.as_ptr().cast())
            }
            OpField::Fixed(fixed) => self.arm_to_vc(*fixed),
            OpField::Hole(hole) => match hole {
                Hole::End => unreachable!(),
                Hole::Void => {
                    if let OpField::Fixed(len) = len {
                        assert!(
                            (*len as usize) < self.void_size,
                            "Dst=!void, but transfer length is greater than void_size"
                        );
                        self.ptr_to_vc(self.void.as_ptr())
                    } else {
                        panic!("Dst=!void requires fixed-length transfer")
                    }
                }
                Hole::Param | Hole::Nil => *hole as u32,
            },
            OpField::OpRefIndirect(op_ref) => {
                let nn = self.allocate_op_ref_indirection(*op_ref);
                self.ptr_to_vc(nn.as_ptr().cast())
            }
            _ => unreachable!(),
        };
        let txfr_len = match len {
            OpField::Fixed(fixed) => *fixed,
            OpField::Hole(hole) => match hole {
                Hole::End => unreachable!(),
                Hole::Void => unreachable!(),
                Hole::Param | Hole::Nil => *hole as u32,
            },
            _ => unreachable!(),
        };
        let nextconbk = match nxt {
            OpField::Fixed(fixed) => self.arm_to_vc(*fixed),
            OpField::Hole(hole) => match hole {
                Hole::End => 0,
                Hole::Void => unreachable!(),
                Hole::Param | Hole::Nil => *hole as u32,
            },
            OpField::OpRef(op_ref) => {
                let nn = self.resolve_op_ref(*op_ref);
                self.ptr_to_vc(nn.as_ptr().cast())
            }
            _ => unreachable!(),
        };

        let mut ti = LocalRegisterCopy::new(0);
        ti.write(
            TI::NO_WIDE_BURSTS::SET
            + TI::BURST_LENGTH::CLEAR
            // + TI::BURST_LENGTH::SET
            + TI::WAIT_RESP::SET
                + TI::DEST_INC::SET + TI::SRC_INC::SET,
        );
        if nextconbk == 0 {
            ti.modify(TI::INTEN::SET);
        }

        let cb = CB {
            ti,
            source_ad,
            dest_ad,
            txfr_len,
            stride: 0, // IGNORE
            nextconbk,
            pad: [0u32; 2], // IGNORE,
        };
        // println!("{cb:08x?}");
        cb
    }
}
impl Frame for DeviceFrame {
    fn new(op_count: usize, max_void: usize) -> Self {
        // let arena = Bump::with_capacity(allocation);
        // println!("executive: allocated arena");
        // arena.set_allocation_limit(Some(allocation));
        let (op_layout, stride) = Layout::new::<CB>()
            .repeat(op_count)
            .expect("should not overflow");
        assert_eq!(
            stride,
            size_of::<CB>(),
            "stride in layout is not equal to CB size"
        );
        // println!("executive: op layout = {layout:?}");
        let op_arena = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(op_layout) })
            .expect("OOM")
            .cast();
        // println!("executive: allocated op_arena = {op_arena:?}");
        let void_layout = Layout::from_size_align(max_void, 4).unwrap();
        let void = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(void_layout) })
            .expect("OOM")
            .cast();
        // println!("executive: allocated void = {void:?}");

        let chunk_map = Vec::new();
        // println!("executive: allocated chunk_map");
        let symbol_map = HashMap::new();
        // println!("executive: allocated symbol_map");
        let routine_map = HashMap::new();
        // println!("executive: allocated routine_map");

        Self {
            chunk_map,
            symbol_map,
            routine_map,
            op_count,
            ops_loaded: 0,
            op_layout,
            op_arena,
            void,
            void_size: max_void,
            void_layout,
        }
    }

    fn load_chunk(
        &mut self,
        symbol: Option<&str>,
        flags: u32,
        layout: core::alloc::Layout,
        backing: Option<&[u8]>,
    ) -> NonNull<u8> {
        // let nn = self.arena.alloc_layout(layout);
        let nn = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(layout) }).expect("OOM");
        // println!("allocated chunk {} = {nn:?}", self.chunk_map.len());
        if let Some(symbol) = symbol {
            self.symbol_map
                .insert(symbol.to_string(), (nn, layout.size()));
        }
        assert_eq!(flags, 0, "unsupported flags: {flags:08x}");
        if let Some(backing) = backing {
            assert_eq!(backing.len(), layout.size());
            for (i, &b) in backing.iter().enumerate() {
                // TODO: this is a slow AF copy
                unsafe { nn.add(i).write_volatile(b) }
            }
        }
        self.chunk_map.push((nn, layout));
        nn
    }

    fn load_ops<I: IntoIterator<Item = sulfur::dilf::Op>>(&mut self, ops: I) {
        for op in ops.into_iter() {
            // println!("op_idx={op_idx}, op_count={}", self.op_count);
            assert!(self.ops_loaded < self.op_count);
            // SAFETY: the allocation is sized for op_count CB's, so we're not going to
            // overrun the array.
            let op_mem: NonNull<CB> = unsafe { self.op_arena.add(self.ops_loaded) };
            let cb = self.translate_op(op);
            // println!("op {} -> {cb:08x?}", self.ops_loaded);
            // SAFETY: `op_arena` is properly aligned for values of type CB, and `add()`
            // will produce a pointer that is equally aligned, since we check that the stride of
            // the layout is equal to the CB size. Furthermore, we the write is valid.
            unsafe { op_mem.write_volatile(cb) };
            self.ops_loaded += 1;
        }
    }

    fn map_routine(&mut self, name: &str, op_idx: usize) {
        assert!(op_idx < self.op_count);
        self.routine_map.insert(name.to_string(), op_idx);
    }
}

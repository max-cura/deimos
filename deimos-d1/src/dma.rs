//! Notes:
//!  Although DMA uses 34-bit addresses, we can't programmatically write the upper 2 bits from DMA
//!  since those are embedded in bitfields in one of the other registers.

use core::{alloc::Layout, ptr::NonNull};

use sulfur::{
    dilf::{DataRef, Executor, Frame, Hole, Op, OpField, OpFieldId, OpFieldRef},
    println,
};
use tock_registers::LocalRegisterCopy;

use alloc::{
    string::{String, ToString as _},
    vec::Vec,
};
use hashbrown::HashMap;

mod registers {
    pub const DRQ_SRAM: u8 = 0;
    pub const DRQ_DRAM: u8 = 1;

    tock_registers::register_bitfields![u32,
        pub Configuration [
            // IGNORE lol
            BMODE_SEL OFFSET(30) NUMBITS(1) [
                Normal = 0,
                BMODE = 1,
            ],
            DMA_DEST_DATA_WIDTH OFFSET(25) NUMBITS(2) [
                _8 = 0b00,
                _16 = 0b01,
                _32 = 0b10,
                _64 = 0b11,
            ],
            DMA_DEST_ADDR_MODE OFFSET(24) NUMBITS(1) [
                Linear = 0,
                Io = 1,
            ],
            DMA_DEST_BLOCK_SIZE OFFSET(22) NUMBITS(2) [
                _1 = 0b00,
                _4 = 0b01,
                _8 = 0b10,
                _16 = 0b11,
            ],
            DMA_DEST_DRQ_TYPE OFFSET(16) NUMBITS(6) [],

            DMA_SRC_DATA_WITDH OFFSET(9) NUMBITS(2) [
                _8 = 0b00,
                _16 = 0b01,
                _32 = 0b10,
                _64 = 0b11,
            ],
            DMA_SRC_ADDR_MODE OFFSET(8) NUMBITS(1) [
                Linear = 0,
                Io = 1,
            ],
            DMA_SRC_BLOCK_SIZE OFFSET(6) NUMBITS(2) [
                _1 = 0b00,
                _4 = 0b01,
                _8 = 0b10,
                _16 = 0b11,
            ],
            DMA_SRC_DRQ_TYPE OFFSET(16) NUMBITS(6) [],
        ],
        pub Parameter [
            HIGH_DEST_AD OFFSET(18) NUMBITS(2),
            HIGH_SRC_AD OFFSET(16) NUMBITS(2),
            WAIT_CLOCK_CYCLES OFFSET(0) NUMBITS(8)
        ],
    ];
}

#[derive(Debug)]
#[repr(C, align(4))]
struct CB {
    configuration: LocalRegisterCopy<u32, registers::Configuration::Register>,
    source_ad_lo32: u32,
    dest_ad_lo32: u32,
    byte_counter: u32,
    parameter: LocalRegisterCopy<u32, registers::Parameter::Register>,
    link: Link,
}
impl CB {
    pub fn set_source_ad(&mut self, addr: u64) -> &mut Self {
        assert_eq!(
            addr & 0x0000_0003_ffff_ffff,
            0,
            "Address must be at most 34 bits"
        );
        let lo32 = (addr & 0xffff_ffff) as u32;
        let hi2 = ((addr & 0x3_0000_0000) >> 32) as u32;
        self.parameter
            .write(registers::Parameter::HIGH_SRC_AD.val(hi2));
        self.source_ad_lo32 = lo32;
        self
    }
    pub fn set_dest_ad(&mut self, addr: u64) -> &mut Self {
        assert_eq!(
            addr & 0x0000_0003_ffff_ffff,
            0,
            "Address must be at most 34 bits"
        );
        let lo32 = (addr & 0xffff_ffff) as u32;
        let hi2 = ((addr & 0x3_0000_0000) >> 32) as u32;
        self.parameter
            .write(registers::Parameter::HIGH_DEST_AD.val(hi2));
        self.dest_ad_lo32 = lo32;
        self
    }
    pub fn set_link(&mut self, addr: u64) -> &mut Self {
        self.link = Link::from_addr(addr);
        self
    }
}

/// DMA CB link field.
#[derive(Debug)]
#[repr(transparent)]
struct Link(u32);
impl Link {
    /// Encode address for DMA CB link field.
    ///
    /// Address space is 34 bits, and address must further be 4-byte aligned.
    pub fn from_addr(addr: u64) -> Self {
        assert_eq!(
            addr & 0x0000_0003_ffff_fffc,
            0,
            "Address will not fit in control-blink link field."
        );
        let x: u32 = ((addr >> 2) & 0xffff_ffff) as u32;
        Self(x.rotate_left(2))
    }

    /// Value of link field that indicates the final DMA CB.
    fn end() -> Self {
        // On the system memory map, this space isn't listed as assigned to anything in particular
        Self(0xffff_f800)
    }
}

pub struct DeviceFrame {
    chunk_map: Vec<(NonNull<u8>, Layout)>,
    symbol_map: HashMap<String, (NonNull<u8>, usize)>,
    routine_map: HashMap<String, usize>,
    op_count: usize,
    op_stride: usize,
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

    fn resolve_data_ref(&self, data_ref: DataRef) -> NonNull<u8> {
        let (chunk_base, chunk_layout) = self
            .chunk_map
            .get(data_ref.chunk as usize)
            .expect("data_ref.chunk should be in-range");
        assert!((data_ref.offset as usize) < chunk_layout.size());
        unsafe { chunk_base.add(data_ref.offset as usize) }
    }
    fn resolve_op_field_ref(&self, op_field_ref: OpFieldRef) -> NonNull<u32> {
        let cb_ptr = self.resolve_op_ref(op_field_ref.op);
        // assert!((op_field_ref.op as usize) < self.op_count);
        // let cb_ptr = unsafe { self.op_arena.add(op_field_ref.op as usize) }.cast::<u32>();
        unsafe {
            cb_ptr
                .cast::<u32>()
                .add(self.op_field_offset(op_field_ref.field_id))
                .byte_add(op_field_ref.offset as usize)
        }
    }
    fn resolve_op_ref(&self, op_ref: u32) -> NonNull<CB> {
        assert!((op_ref as usize) < self.op_count);
        unsafe { self.op_arena.byte_add(op_ref as usize * self.op_stride) }
    }
    fn fixed_to_dma(&self, fixed: u64) -> u64 {
        fixed
    }
    fn ptr_to_dma(&self, ptr: *mut u8) -> u64 {
        ptr.expose_provenance() as u64
    }

    fn alloc_indirection(&mut self) -> NonNull<u32> {
        let layout = Layout::new::<u32>();
        let nn = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(layout) }).expect("OOM");
        self.chunk_map.push((nn, layout));
        nn.cast()
    }
    fn allocate_op_field_ref_indirection(&mut self, op_field_ref: OpFieldRef) -> NonNull<u32> {
        let nn = self.resolve_op_field_ref(op_field_ref);
        let as_dma = self.ptr_to_dma(nn.as_ptr().cast());
        let ind_ptr = self.alloc_indirection();
        // println!("allocated OpFieldRef indirection for {op_field_ref:?} = {ind_ptr:?}");
        unsafe { ind_ptr.write_volatile(as_dma as u32) };
        ind_ptr
    }
    fn allocate_data_ref_indirection(&mut self, data_ref: DataRef) -> NonNull<u32> {
        let nn = self.resolve_data_ref(data_ref);
        let as_dma = self.ptr_to_dma(nn.as_ptr());
        let ind_ptr = self.alloc_indirection();
        // println!("allocated DataRef indirection for {data_ref:?} = {ind_ptr:?}");
        unsafe { ind_ptr.write_volatile(as_dma as u32) };
        ind_ptr
    }
    fn allocate_op_ref_indirection(&mut self, op_ref: u32) -> NonNull<u32> {
        let nn = self.resolve_op_ref(op_ref);
        let as_dma = self.ptr_to_dma(nn.as_ptr().cast());
        let ind_ptr = self.alloc_indirection();
        // println!("allocated OpRef indirection for {op_ref} = {ind_ptr:?}");
        unsafe { ind_ptr.write_volatile(as_dma as u32) };
        ind_ptr
    }

    fn translate_op(&mut self, op: Op) -> CB {
        let dst = op.dst();
        let src = op.src();
        let len = op.len();
        let nxt = op.nxt();

        // Note to self: for non-concrete source/dest addresses, need to set hi2 bits properly
        //

        let dest_ad: u64 = match dst {
            OpField::DataRef(data_ref) => {
                let nn = self.resolve_data_ref(*data_ref);
                self.ptr_to_dma(nn.as_ptr())
            }
            OpField::OpFieldRef(op_field_ref) => {
                let nn = self.resolve_op_field_ref(*op_field_ref);
                self.ptr_to_dma(nn.as_ptr().cast())
            }
            OpField::Fixed(fixed) => self.fixed_to_dma(*fixed as u64),
            OpField::Hole(hole) => match hole {
                Hole::End => unreachable!(),
                Hole::Void => {
                    if let OpField::Fixed(len) = len {
                        assert!(
                            (*len as usize) < self.void_size,
                            "Src=!void, but transfer length is greater than void_size"
                        );
                        self.ptr_to_dma(self.void.as_ptr())
                    } else {
                        panic!("Src=!void requires fixed-length transfer")
                    }
                }
                Hole::Param | Hole::Nil => *hole as u64,
            },
            _ => unreachable!(),
        };
        let source_ad = match src {
            OpField::DataRef(data_ref) => {
                let nn = self.resolve_data_ref(*data_ref);
                self.ptr_to_dma(nn.as_ptr())
            }
            OpField::DataRefIndirect(data_ref) => {
                let nn = self.allocate_data_ref_indirection(*data_ref);
                self.ptr_to_dma(nn.as_ptr().cast())
            }
            OpField::OpFieldRef(op_field_ref) => {
                let nn = self.resolve_op_field_ref(*op_field_ref);
                self.ptr_to_dma(nn.as_ptr().cast())
            }
            OpField::OpFieldRefIndirect(op_field_ref) => {
                let nn = self.allocate_op_field_ref_indirection(*op_field_ref);
                self.ptr_to_dma(nn.as_ptr().cast())
            }
            OpField::Fixed(fixed) => self.fixed_to_dma(*fixed as u64),
            OpField::Hole(hole) => match hole {
                Hole::End => unreachable!(),
                Hole::Void => {
                    if let OpField::Fixed(len) = len {
                        assert!(
                            (*len as usize) < self.void_size,
                            "Dst=!void, but transfer length is greater than void_size"
                        );
                        self.ptr_to_dma(self.void.as_ptr())
                    } else {
                        panic!("Dst=!void requires fixed-length transfer")
                    }
                }
                Hole::Param | Hole::Nil => *hole as u64,
            },
            OpField::OpRefIndirect(op_ref) => {
                let nn = self.allocate_op_ref_indirection(*op_ref);
                self.ptr_to_dma(nn.as_ptr().cast())
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
            OpField::Fixed(fixed) => Link::from_addr(self.fixed_to_dma(*fixed as u64)),
            OpField::Hole(hole) => match hole {
                Hole::End => Link::end(),
                Hole::Void => unreachable!(),
                Hole::Param | Hole::Nil => Link::from_addr(*hole as u64),
            },
            OpField::OpRef(op_ref) => {
                let nn = self.resolve_op_ref(*op_ref);
                Link::from_addr(self.ptr_to_dma(nn.as_ptr().cast()))
            }
            _ => unreachable!(),
        };

        todo!()
    }
}
impl Frame for DeviceFrame {
    fn new(op_count: usize, max_void: usize) -> Self
    where
        Self: Sized,
    {
        let (op_layout, op_stride) = Layout::new::<CB>()
            .repeat(op_count)
            .expect("should not overflow");
        println!("DeviceFrame: op_stride={op_stride}");
        let op_arena = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(op_layout) })
            .expect("OOM")
            .cast();

        let void_layout = Layout::from_size_align(max_void, 4).unwrap();
        let void = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(void_layout) })
            .expect("OOM")
            .cast();

        let chunk_map = Vec::new();
        let symbol_map = HashMap::new();
        let routine_map = HashMap::new();

        Self {
            chunk_map,
            symbol_map,
            routine_map,
            op_count,
            op_layout,
            op_stride,
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
        layout: Layout,
        backing: Option<&[u8]>,
    ) -> NonNull<u8> {
        let nn = NonNull::new(unsafe { alloc::alloc::alloc_zeroed(layout) }).expect("OOM");
        if let Some(symbol) = symbol {
            self.symbol_map
                .insert(symbol.to_string(), (nn, layout.size()));
        }
        assert_eq!(flags, 0, "unsupported flags: {flags:08x}");
        if let Some(backing) = backing {
            assert_eq!(backing.len(), layout.size());
            for (i, &b) in backing.iter().enumerate() {
                unsafe { nn.add(i).write_volatile(b) }
            }
        }
        self.chunk_map.push((nn, layout));
        nn
    }

    fn load_ops<I: IntoIterator<Item = sulfur::dilf::Op>>(&mut self, ops: I) {
        for (op_idx, op) in ops.into_iter().enumerate() {
            assert!(op_idx < self.op_count);
            let op_mem: NonNull<CB> = unsafe { self.op_arena.byte_add(op_idx * self.op_stride) };
            let cb = self.translate_op(op);
            unsafe { op_mem.write_volatile(cb) };
        }
    }

    fn map_routine(&mut self, name: &str, op_idx: usize) {
        assert!(op_idx < self.op_count);
        self.routine_map.insert(name.to_string(), op_idx);
    }
}

pub struct Channel {
    channel_no: usize,
}
impl Executor for Channel {
    type Frame = DeviceFrame;

    fn execute(&mut self, frame: &mut Self::Frame, routine: &str) -> sulfur::Timing {
        todo!()
    }
}

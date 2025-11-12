use core::{alloc::Layout, ptr::NonNull};

use sulfur::{
    dilf::{Frame, Op},
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
    fn translate_op(&mut self, op: Op) -> CB {
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

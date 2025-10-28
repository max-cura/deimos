use core::{alloc::Layout, ptr::NonNull};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Dilf32Header {
    pub magic: [u8; 8],
    pub arch: u16,
    pub version: u16,
    pub flags: u32,
    pub code: SegmentSpec,
    pub data: SegmentSpec,
    pub routine_map: SegmentSpec,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SegmentSpec {
    pub offset: u32,
    pub len: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ChunkSpec {
    pub symbol_ref_offset: u32,
    pub flags: u32,
    pub chunk_offset: u32,
    pub file_size: u32,
    pub mem_size: u32,
    pub mem_align: u32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Op {
    pub flags: u32,
    pub dst: Dst,
    pub src: Src,
    pub len: Len,
    pub nxt: Nxt,
}
pub const OP_FLAGS_DST_OFFSET: u32 = 0x0;
pub const OP_FLAGS_SRC_OFFSET: u32 = 0x4;
pub const OP_FLAGS_LEN_OFFSET: u32 = 0x8;
pub const OP_FLAGS_NXT_OFFSET: u32 = 0xc;
impl Op {
    pub fn dst(&self) -> OpField<'_> {
        let this = &self.dst;
        match (self.flags >> OP_FLAGS_DST_OFFSET) & 0xf {
            0 => OpField::DataRef(unsafe { &this.data_ref }),
            1 => unreachable!("unsupported type for Dst: DataRefIndirect"),
            2 => OpField::OpFieldRef(unsafe { &this.op_ref_field }),
            3 => unreachable!("unsupported type for Dst: OpFieldRefIndirect"),
            4 => OpField::Fixed(unsafe { &this.fixed }),
            5 => OpField::Hole(unsafe { &this.hole }),
            6 => unreachable!("unsupported type for Dst: OpRef"),
            x => unreachable!("unknown type for Dst: {x}"),
        }
    }
    pub fn src(&self) -> OpField<'_> {
        let this = &self.src;
        match (self.flags >> OP_FLAGS_SRC_OFFSET) & 0xf {
            0 => OpField::DataRef(unsafe { &this.data_ref }),
            1 => OpField::DataRefIndirect(unsafe { &this.data_ref_indirect }),
            2 => OpField::OpFieldRef(unsafe { &this.op_ref_field }),
            3 => OpField::OpFieldRefIndirect(unsafe { &this.op_ref_field_indirect }),
            4 => OpField::Fixed(unsafe { &this.fixed }),
            5 => OpField::Hole(unsafe { &this.hole }),
            6 => OpField::OpRefIndirect(unsafe { &this.op_ref_indirect }),
            x => unreachable!("unknown type for Src: {x}"),
        }
    }
    pub fn len(&self) -> OpField<'_> {
        let this = &self.len;
        match (self.flags >> OP_FLAGS_LEN_OFFSET) & 0xf {
            0 => unreachable!("unsupported type for Len: DataRef"),
            1 => unreachable!("unsupported type for Len: DataRefIndirect"),
            2 => unreachable!("unsupported type for Len: OpFieldRef"),
            3 => unreachable!("unsupported type for Len: OpFieldRefIndirect"),
            4 => OpField::Fixed(unsafe { &this.fixed }),
            5 => OpField::Hole(unsafe { &this.hole }),
            6 => unreachable!("unsupported type for Len: OpRef"),
            x => unreachable!("unknown type for Len: {x}"),
        }
    }
    pub fn nxt(&self) -> OpField<'_> {
        let this = &self.nxt;
        match (self.flags >> OP_FLAGS_NXT_OFFSET) & 0xf {
            0 => unreachable!("unsupported type for Nxt: DataRef"),
            1 => unreachable!("unsupported type for Nxt: DataRefIndirect"),
            2 => unreachable!("unsupported type for Nxt: OpFieldRef"),
            3 => unreachable!("unsupported type for Nxt: OpFieldRefIndirect"),
            4 => OpField::Fixed(unsafe { &this.fixed }),
            5 => OpField::Hole(unsafe { &this.hole }),
            6 => OpField::OpRef(unsafe { &this.op_ref }),
            x => unreachable!("unknown type for Nxt: {x}"),
        }
    }
}
#[derive(Copy, Clone)]
pub enum OpField<'op> {
    DataRef(&'op DataRef),
    DataRefIndirect(&'op DataRef),
    OpFieldRef(&'op OpFieldRef),
    OpFieldRefIndirect(&'op OpFieldRef),
    Fixed(&'op u32),
    Hole(&'op Hole),
    OpRef(&'op u32),
    OpRefIndirect(&'op u32),
}
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Hole {
    End = 0,
    Void = 1,
    Param = 2,
    Nil = 3,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union Dst {
    pub data_ref: DataRef,
    pub op_ref_field: OpFieldRef,
    pub fixed: u32,
    pub hole: Hole,
}
impl Dst {
    pub fn data_ref(chunk: usize, offset: usize) -> Self {
        Self {
            data_ref: DataRef {
                chunk: chunk as u32,
                offset: offset as u32,
            },
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union Src {
    pub data_ref: DataRef,
    pub data_ref_indirect: DataRef,
    pub op_ref_field: OpFieldRef,
    pub op_ref_field_indirect: OpFieldRef,
    pub fixed: u32,
    pub hole: Hole,
    pub op_ref_indirect: u32,
}
impl Src {
    pub fn data_ref(chunk: usize, offset: usize) -> Self {
        Self {
            data_ref: DataRef {
                chunk: chunk as u32,
                offset: offset as u32,
            },
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union Len {
    pub fixed: u32,
    pub hole: Hole,
}
impl Len {
    pub fn fixed(size: usize) -> Self {
        Self {
            fixed: size.try_into().unwrap(),
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union Nxt {
    pub op_ref: u32,
    pub fixed: u32,
    pub hole: Hole,
}
impl Nxt {
    pub fn end() -> Self {
        Self { hole: Hole::End }
    }
    pub fn op_ref(i: usize) -> Self {
        Self { op_ref: i as u32 }
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DataRef {
    pub chunk: u32,
    pub offset: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpFieldRef {
    pub op: u32,
    pub field_id: OpFieldId,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OpFieldId {
    Dst = 0,
    Src = 1,
    Len = 2,
    Nxt = 3,
}

pub trait Loader {
    // NOTE: THIS MUST BE CALLED IN THE CORRECT ORDER
    fn load_chunk(
        &mut self,
        symbol_ref: Option<&str>,
        flags: u32,
        layout: Layout,
        backing: Option<&[u8]>,
    ) -> NonNull<u8>;
    fn load_ops<I: IntoIterator<Item = Op>>(&mut self, ops: I);
    fn map_routine(&mut self, name: &str, op_idx: usize);
}

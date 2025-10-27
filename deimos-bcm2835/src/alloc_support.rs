unsafe extern "C" {
    static __exec_end: [u32; 0];
}

#[global_allocator]
static HEAP: embedded_alloc::TlsfHeap = embedded_alloc::TlsfHeap::empty();
pub fn heap_init() {
    // XXX: not sure if this is well-defined or not.
    // let heap_begin = (&raw const __exec_end) as usize;
    let heap_begin = 0x0200_0000;
    let heap_end = 0x0800_0000;
    unsafe {
        HEAP.init(heap_begin, heap_end - heap_begin);
    }
}

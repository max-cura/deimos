use sulfur::Flushable;

#[derive(Debug)]
pub struct UartProxy;

impl core::fmt::Write for UartProxy {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut device =
            unsafe { crate::uart::Device::new(crate::uart::RegisterBlock::from_addr(0x1c28000)) };
        for &b in s.as_bytes() {
            while !device.status().can_write() {}
            device.write_byte(b);
        }
        Ok(())
    }
}

impl Flushable for UartProxy {
    fn flush(&mut self) {
        let mut device =
            unsafe { crate::uart::Device::new(crate::uart::RegisterBlock::from_addr(0x1c28000)) };
        device.flush();
    }
}

sulfur::set_impl!(UartProxy);

use d1_pac::UART0;
use sulfur::Flushable;

#[derive(Debug)]
pub struct UartProxy;

impl core::fmt::Write for UartProxy {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let uart = unsafe { UART0::steal() };
        for &b in s.as_bytes() {
            while uart.usr().read().tfnf().bit_is_clear() {}
            uart.thr().write(|w| w.thr().variant(b));
        }
        Ok(())
    }
}

impl Flushable for UartProxy {
    fn flush(&mut self) {
        let uart = unsafe { UART0::steal() };
        while uart.lsr().read().temt().bit_is_clear() {}
        while uart.usr().read().busy().is_busy() {}
    }
}

sulfur::set_impl!(UartProxy);

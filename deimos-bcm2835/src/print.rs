use core::fmt::Write as _;

use crate::arch::dsb;
use bcm2835_lpa::Peripherals;
use sulfur::Flushable;

#[derive(Debug)]
pub struct UartProxy;

impl core::fmt::Write for UartProxy {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let peri = unsafe { Peripherals::steal() };
        dsb();
        for &b in s.as_bytes() {
            while !peri.UART1.stat().read().tx_ready().bit_is_set() {}
            peri.UART1.io().write(|w| unsafe { w.data().bits(b) });
        }
        dsb();
        Ok(())
    }
}

impl Flushable for UartProxy {
    fn flush(&mut self) {
        let peri = unsafe { Peripherals::steal() };
        crate::uart::flush_tx_fifo(&peri.UART1);
    }
}

sulfur::set_impl!(UartProxy);

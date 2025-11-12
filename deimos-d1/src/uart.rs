use core::ops::Deref;

use d1_pac::{CCU, GPIO};

pub trait UartPinMuxHelper {
    const UART_NO: u8;
    fn mux(&self, gpio: &GPIO);
}
impl UartPinMuxHelper for d1_pac::UART0 {
    const UART_NO: u8 = 0;
    fn mux(&self, gpio: &GPIO) {
        gpio.pb_cfg1()
            .modify(|_, w| w.pb8_select().uart0_tx().pb9_select().uart0_rx());
        // BUG: name is wrong - this is pb8_pull(), not pc8_pull()
        gpio.pb_pull0().modify(|_, w| w.pc8_pull().pull_up());
    }
}

pub fn init<U>(ccu: &CCU, gpio: &GPIO, uart: &U, apb_clock: u32, baud_rate: u32)
where
    U: UartPinMuxHelper + Deref<Target = d1_pac::uart::RegisterBlock>,
{
    uart.mux(gpio);

    ccu.uart_bgr()
        .modify(|_, w| w.uart_gating(U::UART_NO).pass());
    ccu.uart_bgr()
        .modify(|_, w| w.uart_rst(U::UART_NO).deassert());

    uart.ier().write(|w| unsafe { w.bits(0) });
    uart.fcr().write(|w| unsafe { w.bits(0) });
    uart.mcr().write(|w| unsafe { w.bits(0) });
    uart.lcr().write(|w| unsafe { w.bits(0) });

    let prescaler = apb_clock / 16 / baud_rate;
    let prescaler_low = ((prescaler & 0x0000_00ff) >> 0) as u8;
    let prescaler_high = ((prescaler & 0x0000_ff00) >> 8) as u8;

    uart.lcr().write(|w| w.dlab().divisor_latch());
    uart.dll().write(|w| w.dll().variant(prescaler_low));
    uart.dlh().write(|w| w.dlh().variant(prescaler_high));
    uart.lcr().write(|w| w.dlab().rx_buffer());
    uart.fcr().write(|w| w.fifoe().set_bit());
    #[rustfmt::skip]
    uart.lcr().write(|w| w.dls().eight().pen().disabled().stop().one().eps().odd());
}

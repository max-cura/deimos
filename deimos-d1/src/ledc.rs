use d1_pac::{CCU, GPIO, LEDC};

pub fn init(gpio: &GPIO, ccu: &CCU) {
    gpio.pc_cfg0().modify(|_, w| w.pc0_select().ledc_do());
    ccu.ledc_clk().modify(|_, w| w.clk_gating().on());
    ccu.ledc_bgr().modify(|_, w| w.rst().deassert());
    ccu.ledc_bgr().modify(|_, w| w.gating().pass());
}

pub fn set(color: u32) {
    debug_assert_eq!(color & 0xff_00_00_00, 0, "expected RGB888 color");
    let _color = color & 0x00_ff_ff_ff;

    let ledc = unsafe { LEDC::steal() };
    while ledc.ledc_ctrl().read().ledc_en().is_enable() {}
    ledc.ledc_data().write(|w| unsafe { w.bits(color) });
    ledc.ledc_ctrl()
        .modify(|_, w| w.total_data_length().variant(1).led_rgb_mode().rgb());
    ledc.ledc_dma_ctrl()
        .modify(|_, w| w.ledc_dma_en().disable());
    ledc.ledc_ctrl().modify(|_, w| w.ledc_en().enable());
}

use d1_pac::CCU;

pub fn init(ccu: &CCU) {
    // Set APB frequency: 200MHz
    ccu.apb1_clk()
        .modify(|_, w| w.factor_m().variant(2).factor_n().n1());
    // ... using Peri PLL with frequency 600MHz
    ccu.apb1_clk().modify(|_, w| w.clk_src_sel().pll_peri_1x());
}

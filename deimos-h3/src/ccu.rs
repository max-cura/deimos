// XXX: For bus_clk_gating_YYY, we have mask=0, pass=1
//      For bus_soft_rst_YYY, we have assert=0, deassert=1
//      The effect is basically the same, but the semantics are different
tock_registers::register_bitfields![
    u32,
    pub BusClockGatingGroup0 [
        HSTMR_GATING OFFSET(19) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        TS_GATING OFFSET(18) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        EMAC_GATING OFFSET(17) NUMBITS(1) [ Mask = 0, Pass = 1],
        DMA_GATING OFFSET(6) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        CE_GATING OFFSET(5) NUMBITS(1) [ Mask = 0, Pass = 1 ],
    ],
    pub BusClockGatingGroup1 [
        SPINLOCK_GATING OFFSET(22) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        MSGBOX_GATING OFFSET(21) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        GPU_GATING OFFSET(20) NUMBITS(1) [ Mask = 0, Pass = 1 ],
    ],
    pub BusClockGatingGroup2 [
        PIO_GATING OFFSET(5) NUMBITS(1) [ Mask = 0, Pass = 1 ],
    ],
    pub BusClockGatingGroup3 [
        SCR_GATING OFFSET(20) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        UART3_GATING OFFSET(19) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        UART2_GATING OFFSET(18) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        UART1_GATING OFFSET(17) NUMBITS(1) [ Mask = 0, Pass = 1 ],
        UART0_GATING OFFSET(16) NUMBITS(1) [ Mask = 0, Pass = 1 ],
    ],
    pub BusClockGatingGroup4 [
        EPHY_GATING OFFSET(0) NUMBITS(1) [ Mask=  0, Pass = 1 ],
    ],
    pub BusSoftResetGroup0 [
        HSTMR_RST OFFSET(19) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        TS_RST OFFSET(18) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        EMAC_RST OFFSET(17) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        DMA_RST OFFSET(6) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        CE_RST OFFSET(5) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
    ],
    pub BusSoftResetGroup1 [
        SPINLOCK_RST OFFSET(22) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        MSGBOX_RST OFFSET(21) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        GPU_RST OFFSET(20) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
    ],
    pub BusSoftResetGroup2 [
        // XXX: This is different from the gating group 2; this is also the ONLY field of
        //      reset group 2 that is described
        EPHY_RST OFFSET(2) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
    ],
    pub BusSoftResetGroup3 [ // XXX: COMPLETED
        I2S_PCM_2_RST OFFSET(14) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        I2S_PCM_1_RST OFFSET(13) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        I2S_PCM_0_RST OFFSET(12) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        THS_RST OFFSET(8) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        OWA_RST OFFSET(1) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        AC_RST OFFSET(0) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
    ],
    pub BusSoftResetGroup4 [ // XXX: COMPLETED
        SCR_RST OFFSET(20) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        UART3_RST OFFSET(19) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        UART2_RST OFFSET(18) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        UART1_RST OFFSET(17) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        UART0_RST OFFSET(16) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        TWI2_RST OFFSET(2) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        TWI1_RST OFFSET(1) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
        TWI0_RST OFFSET(0) NUMBITS(1) [ Assert = 0, Deassert = 1 ],
    ],
];

tock_registers::peripheral! {
    #[real(Ccu)]
    /// NOTES:
    ///  1. Release reset signal BEFORE releasing clock gating [1]
    ///
    /// [1] Allwinner_H3_Datasheet_V1.2.pdf §4.3.6.4 p142
    pub CcuTrait {
        0x0060 => bus_clk_gating_group0: BusClockGatingGroup0::Register { Read, Write },
        0x0064 => bus_clk_gating_group1: BusClockGatingGroup1::Register { Read, Write },
        0x0068 => bus_clk_gating_group2: BusClockGatingGroup2::Register { Read, Write },
        0x006c => bus_clk_gating_group3: BusClockGatingGroup3::Register { Read, Write },
        0x0070 => bus_clk_gating_group4: BusClockGatingGroup4::Register { Read, Write },
        0x02c0 => bus_soft_reset_group0: BusSoftResetGroup0::Register { Read, Write },
        0x02c4 => bus_soft_reset_group1: BusSoftResetGroup1::Register { Read, Write },
        0x02c8 => bus_soft_reset_group2: BusSoftResetGroup2::Register { Read, Write },
        // XXX: there's a gap at 0x02cc, for some odd reason
        0x02d0 => bus_soft_reset_group3: BusSoftResetGroup3::Register { Read, Write },
        0x02d4 => bus_soft_reset_group4: BusSoftResetGroup4::Register { Read, Write },
    }
}

#[allow(unused)]
pub const CCU_BASE_ADDR: usize = 0x1c20_0000;

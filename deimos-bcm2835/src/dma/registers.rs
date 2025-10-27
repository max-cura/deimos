tock_registers::register_bitfields! {
    // 32-bit registers
    u32,
    /// DMA Control/Status Register
    pub CS [
        RESET OFFSET(31) NUMBITS(1) [],
        ABORT OFFSET(30) NUMBITS(1) [],
        DISDEBUG OFFSET(29) NUMBITS(1) [Nonstop = 1, Stoppable = 0],
        WAIT_FOR_OUTSTANDING_WRITES OFFSET(28) NUMBITS(1) [],
        PANIC_PRIORITY OFFSET(20) NUMBITS(4) [],
        PRIORITY OFFSET(16) NUMBITS(4) [],
        // Clear by using the debug register.
        ERROR OFFSET(8) NUMBITS(1) [],
        /// Indicates that DMA is currently waiting for outstanding writes to be received and is
        /// not transferring data.
        WAITING_FOR_OUSTANDING_WRITES OFFSET(6) NUMBITS(1) [],
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [],
        /// Paused due to: active bit cleared, executing wait cycles, debug_pause, too many
        /// outstanding writes
        PAUSED OFFSET(4) NUMBITS(1) [],
        /// NOTE: If PERMAP is set to 0, then this bit will read as Requesting (1).
        DREQ OFFSET(3) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) [],
        END OFFSET(1) NUMBITS(1) [],
        ACTIVE OFFSET(0) NUMBITS(1) [],
    ],
    /// DMA Transfer Information
    pub TI [
        NO_WIDE_BURSTS OFFSET(26) NUMBITS(1) [],
        WAITS OFFSET(21) NUMBITS(5) [],
        PERMAP OFFSET(16) NUMBITS(5) [],
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
        SRC_WIDTH OFFSET(9) NUMBITS(1) [],
        SRC_INC OFFSET(8) NUMBITS(1) [],
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
        DEST_WIDTH OFFSET(5) NUMBITS(1) [],
        DEST_INC OFFSET(4) NUMBITS(1) [],
        WAIT_RESP OFFSET(3) NUMBITS(1) [],
        TDMODE OFFSET(1) NUMBITS(1) [],
        INTEN OFFSET(0) NUMBITS(1) [],
    ]
}

tock_registers::peripheral! {
    #[real(DynChannel)]
    pub Channel {
        0x00 => cs: CS::Register { Read, Write },
        0x04 => conblk_ad: u32 { Read, Write },
        0x08 => ti: TI::Register { Read, Write },
        0x0c => source_ad: u32 { Read, Write },
        0x10 => dest_ad: u32 { Read, Write },
        0x14 => txfr_len: u32 { Read, Write },
        0x18 => stride: u32 { Read, Write },
        0x1c => nextconbk: u32 { Read, Write },
        0x20 => debug: u32 { Read, Write },
    }
}

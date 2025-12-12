use tock_registers::{Read, RegisterCopy, Write};

tock_registers::register_bitfields![u32,
    ReceiveBuffer [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    TransmitHolding [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    DivisorLatchLow [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    DivisorLatchHigh [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    InterruptEnable [
        PTIME_OFFSET OFFSET(7) NUMBITS(1) [],
        RS485_INT_EN OFFSET(4) NUMBITS(1) [],
        EDSSI OFFSET(3) NUMBITS(1) [],
        ELSI OFFSET(2) NUMBITS(1) [],
        ETBEI OFFSET(1) NUMBITS(1) [],
        ERBFI OFFSET(0) NUMBITS(1) [],
    ],
    InterruptIdentity [
        FIFOS_ENABLED OFFSET(6) NUMBITS(2) [
            Enabled = 0b11,
            Disabled = 0b00,
        ],
        IID OFFSET(0) NUMBITS(4) [
            ModemStatus = 0,
            NoInterruptPending = 1,
            TransmitHoldingRegisterEmpty = 2,
            RS485Interrupt = 3,
            ReceiverDataAvailable = 4,
            ReceiverLineStatus = 6,
            BusyDetect = 7,
            CharacterTimeout = 12,
        ],
    ],
    FifoControl [
        RT OFFSET(6) NUMBITS(2) [
            FifoOne = 0,
            FifoQuarter = 1,
            FifoHalf = 2,
            FifoMinusTwo= 3,
        ],
        TFT OFFSET(4) NUMBITS(2) [
            FifoEmpty = 0,
            FifoTwo = 1,
            FifoQuarter = 2,
            FifoHalf = 3,
        ],
        DMAM OFFSET(3) NUMBITS(1) [],
        XFIFOR OFFSET(2) NUMBITS(1) [],
        RFIFOR OFFSET(1) NUMBITS(1) [],
        FIFOE OFFSET(0) NUMBITS(1) [],
    ],
    LineControl [
        DLAB OFFSET(7) NUMBITS(1) [],
        BC OFFSET(6) NUMBITS(1) [],
        EPS OFFSET(4) NUMBITS(2) [
            Odd = 0b00,
            Even = 0b01,
            ReverseOddOrRS485Bit9Address = 0b10,
            ReverseOddOrRS485Bit9Data = 0b11,
        ],
        PEN OFFSET(3) NUMBITS(1) [],
        STOP OFFSET(2) NUMBITS(1) [
            _1 = 0b0,
            _2 = 0b1,
        ],
        DLEN OFFSET(0) NUMBITS(2) [
            _5Bits = 0b00,
            _6Bits = 0b01,
            _7Bits = 0b10,
            _8Bits = 0b11,
        ]
    ],
    ModemControl [
        UART_FUNCTION OFFSET(6) NUMBITS(2) [
            Uart = 0b00,
            IrDASir= 0b01,
            RS485 = 0b10,
        ],
        AFCE OFFSET(5) NUMBITS(1) [],
        LOOP OFFSET(4) NUMBITS(1) [
            Normal = 0,
            LOOPBACk = 1,
        ],
        RTS OFFSET(1) NUMBITS(1) [],
        CTX OFFSET(0) NUMBITS(1) [],
    ],
    LineStatus [
        RX_FIFO_ERR OFFSET(7) NUMBITS(1) [],
        /// TX FIFO and TX shift register are BOTH empty
        TX_EMPTY OFFSET(6) NUMBITS(1) [],
        /// TX FIFO is empty, but TX shift register may not be.
        TX_FIFO_EMPTY OFFSET(5) NUMBITS(1) [],
        BRK_INT OFFSET(4) NUMBITS(1) [],
        RX_FRAME_ERR OFFSET(3) NUMBITS(1) [],
        RX_PARITY_ERR OFFSET(2) NUMBITS(1) [],
        RX_OVERRUN_ERR OFFSET(1) NUMBITS(1) [],
        RX_DATA_READY OFFSET(0) NUMBITS(1) [],
    ],
    UartStatus [
        RX_FIFO_FULL OFFSET(4) NUMBITS(1) [],
        RX_FIFO_NOT_EMPTY OFFSET(3) NUMBITS(1) [],
        TX_FIFO_EMPTY OFFSET(2) NUMBITS(1) [],
        TX_FIFO_NOT_FULL OFFSET(1) NUMBITS(1) [],
        BUSY OFFSET(1) NUMBITS(1) [],
    ]
];

tock_registers::peripheral! {
    // XXX: There's a few registers (mostly DMA-related) that I left out because they're not
    //      necessary for my purposes at the moment.
    #[real(RegisterBlock)]
    pub Registers {
        0x00 => rbr: ReceiveBuffer::Register { Read },
        0x00 => thr: TransmitHolding::Register { Write },
        0x00 => dll: DivisorLatchLow::Register { Write },
        0x04 => dlh: DivisorLatchHigh::Register { Write },
        0x04 => ier: InterruptEnable::Register { Read, Write },
        0x08 => iir: InterruptIdentity::Register { Read },
        0x08 => fcr: FifoControl::Register { Write },
        0x0c => lcr: LineControl::Register { Read, Write },
        0x10 => mcr: ModemControl::Register { Read, Write },
        0x14 => lsr: LineStatus::Register { Read },
        0x7c => usr: UartStatus::Register { Read },
    }
}

// fn generic_over_registers<R: Registers>(r: &R) -> bool {
//     r.usr().is_set(UartStatus::TX_FIFO_NOT_FULL)
//     r.usr().
// }

pub struct Status {
    usr: RegisterCopy<UartStatus::Register>,
}
impl Status {
    pub fn can_write(&mut self) -> bool {
        self.usr.is_set(UartStatus::TX_FIFO_NOT_FULL)
    }

    // pub fn can_read(&mut self) -> bool {
    //     self.usr.is_set(UartStatus::RX_FIFO_NOT_EMPTY)
    // }
}

pub struct Device {
    registers: RegisterBlock,
}
impl Device {
    pub unsafe fn new(registers: RegisterBlock) -> Self {
        Self { registers }
    }
    pub fn new_init(registers: RegisterBlock, from_apb_freq: u32, baud_rate: u32) -> Self {
        registers.ier().write(0);
        registers.fcr().write(0);
        registers.mcr().write(0);
        registers.lcr().write(0);

        let prescaler = from_apb_freq / 16 / baud_rate;
        let prescaler_low = ((prescaler & 0x00ff) >> 0) as u8;
        let prescaler_high = ((prescaler & 0xff00) >> 8) as u8;

        registers.lcr().write_field(LineControl::DLAB::SET);
        registers
            .dll()
            .write_field(DivisorLatchLow::DATA.val(prescaler_low.into()));
        registers
            .dlh()
            .write_field(DivisorLatchHigh::DATA.val(prescaler_high.into()));
        registers.lcr().write_field(LineControl::DLAB::CLEAR);
        registers.fcr().write_field(FifoControl::FIFOE::SET);
        registers.lcr().write_field(
            LineControl::PEN::CLEAR
                + LineControl::STOP::_1
                + LineControl::DLEN::_8Bits
                + LineControl::EPS::Odd,
        );
        Self { registers }
    }
}
impl Device {
    pub fn status(&mut self) -> Status {
        Status {
            usr: self.registers.usr().extract(),
        }
    }
    pub fn flush(&mut self) {
        while self
            .registers
            .usr()
            .matches_any(&[UartStatus::TX_FIFO_EMPTY::CLEAR])
        {}
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.registers.thr().write(byte.into());
    }

    // fn read_byte(&mut self) -> u8 {
    //     let r = (self.registers.rbr().read() & 0xff).try_into();
    //     unsafe { r.unwrap_unchecked() }
    // }

    // fn enable_tx_int(&mut self, en: bool) {
    //     self.registers.ier().write_field(if en {
    //         InterruptEnable::ETBEI::SET
    //     } else {
    //         InterruptEnable::ETBEI::CLEAR
    //     });
    // }

    // fn enable_rx_int(&mut self, en: bool) {
    //     self.registers.ier().write_field(if en {
    //         InterruptEnable::ERBFI::SET
    //     } else {
    //         InterruptEnable::ERBFI::CLEAR
    //     })
    // }
}

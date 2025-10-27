use crate::mailbox;
use core::fmt::{Display, Formatter};

pub fn query() -> BoardRevision {
    let mut buf = [0u8; 4];
    mailbox::send_message(0x0001_0002, &mut buf);
    BoardRevision::from(u32::from_ne_bytes(buf))
}

pub enum BoardModel {
    A = 0,
    B = 1,
    APlus = 2,
    BPlus = 3,
    _2B = 4,
    Alpha = 5,
    CM1 = 6,
    _3B = 8,
    Zero = 9,
    CM3 = 10,
    ZeroW = 12,
    _3BPlus = 13,
    _3APlus = 14,
}
impl Display for BoardModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::A => "Model A",
                Self::B => "Model B",
                Self::APlus => "Model A+",
                Self::BPlus => "Model B+",
                Self::_2B => "Model 2B",
                Self::Alpha => "Model Alpha",
                Self::CM1 => "Model CM1",
                Self::_3B => "Model 3B",
                Self::Zero => "Model Zero",
                Self::CM3 => "Model CM3",
                Self::ZeroW => "Model Zero W",
                Self::_3BPlus => "Model 3B+",
                Self::_3APlus => "Model 3A+",
            }
        )
    }
}
impl From<u32> for BoardModel {
    fn from(value: u32) -> Self {
        match value {
            0 => BoardModel::A,
            1 => BoardModel::B,
            2 => BoardModel::APlus,
            3 => BoardModel::BPlus,
            4 => BoardModel::_2B,
            5 => BoardModel::Alpha,
            6 => BoardModel::CM1,
            8 => BoardModel::_3B,
            9 => BoardModel::Zero,
            10 => BoardModel::CM3,
            12 => BoardModel::ZeroW,
            13 => BoardModel::_3BPlus,
            14 => BoardModel::_3APlus,
            x => panic!("unknown board model: {}", x),
        }
    }
}
pub enum BoardProcessor {
    Bcm2835 = 0,
    Bcm2836 = 1,
    Bcm2837 = 2,
}
impl Display for BoardProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BoardProcessor::Bcm2835 => "BCM2835",
                BoardProcessor::Bcm2836 => "BCM2836",
                BoardProcessor::Bcm2837 => "BCM2837",
            }
        )
    }
}
impl From<u32> for BoardProcessor {
    fn from(value: u32) -> Self {
        match value {
            0 => BoardProcessor::Bcm2835,
            1 => BoardProcessor::Bcm2836,
            2 => BoardProcessor::Bcm2837,
            x => panic!("unknown processor: {x}"),
        }
    }
}
pub enum BoardManufacturer {
    SonyUk = 0,
    Egoman = 1,
    Embest = 2,
    SonyJapan = 3,
    Stadium = 5,
}
impl Display for BoardManufacturer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SonyUk => write!(f, "Sony UK"),
            Self::Egoman => write!(f, "Egoman"),
            Self::Embest => write!(f, "Embest"),
            Self::SonyJapan => write!(f, "Sony JAPAN"),
            Self::Stadium => write!(f, "Stadium"),
        }
    }
}
impl From<u32> for BoardManufacturer {
    fn from(value: u32) -> Self {
        match value {
            0 => BoardManufacturer::SonyUk,
            1 => BoardManufacturer::Egoman,
            2 => BoardManufacturer::Embest,
            3 => BoardManufacturer::SonyJapan,
            4 => BoardManufacturer::Embest,
            5 => BoardManufacturer::Stadium,
            x => panic!("unknown manufacturer: {x}"),
        }
    }
}
pub struct BoardRevision {
    pub model: BoardModel,
    pub processor: BoardProcessor,
    pub revision_minor: u8,
    pub ram_mb: u16,
    pub manufacturer: BoardManufacturer,
}
impl Display for BoardRevision {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "model:{},proc:{},rev:1.{},ram:{}MiB,manufacturer:{}",
            self.model, self.processor, self.revision_minor, self.ram_mb, self.manufacturer
        )
    }
}
impl From<u32> for BoardRevision {
    fn from(revision: u32) -> Self {
        if (revision & 0x0080_0000) == 0 {
            panic!("<BoardRevision as From<u32>>::from() does not support old-style revisions")
        }

        let model = (revision >> 4) & 0xff;
        let proc = (revision >> 12) & 0xf;
        let rev_minor = revision & 0xf;
        let ram_mb = 256 << ((revision >> 20) & 7);
        let manufacturer = (revision >> 16) & 0xf;
        Self {
            model: BoardModel::from(model),
            processor: BoardProcessor::from(proc),
            revision_minor: rev_minor as u8,
            ram_mb: ram_mb as u16,
            manufacturer: BoardManufacturer::from(manufacturer),
        }
    }
}

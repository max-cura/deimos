use crate::mailbox;
use core::fmt::{Display, Formatter};

pub struct DmaChannels {
    channels: [bool; 16],
}
impl DmaChannels {
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            channels: &self.channels,
            idx: 0,
        }
    }
}
impl Display for DmaChannels {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut any = false;
        let mut from = None;
        for i in 0..16 {
            if !self.channels[i]
                && let Some(from) = from
            {
                if any {
                    write!(f, ", ")?
                }
                if from == i - 1 {
                    write!(f, "{}", from)
                } else {
                    write!(f, "{}-{}", from, i - 1)
                }?;
                any = true
            }
            from = if self.channels[i] {
                from.or(Some(i))
            } else {
                None
            }
        }
        if let Some(from) = from {
            if from == 15 {
                write!(f, "15")
            } else {
                write!(f, "{}-15", from)
            }?
        }
        if !any { write!(f, "<none>") } else { Ok(()) }
    }
}

pub struct Iter<'a> {
    channels: &'a [bool; 16],
    idx: usize,
}
impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < 16 {
            let idx = self.idx;
            self.idx += 1;
            if self.channels[idx] {
                return Some(idx);
            }
        }
        None
    }
}

pub fn query() -> DmaChannels {
    let mut dma = [0u32];
    mailbox::send_message(0x0006_0001, bytemuck::cast_slice_mut(&mut dma));
    let mut channels = DmaChannels {
        channels: [false; 16],
    };
    for i in 0..16 {
        channels.channels[i] = (dma[0] & (1 << i)) != 0;
    }
    channels
}

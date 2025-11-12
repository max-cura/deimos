use core::arch::asm;
use core::ops::Sub;
use core::time::Duration;

/// Returns with a resolution somewhere in the vicinity of \~42\~45ns.
/// According to [this](https://misc0110.net/web/files/riscv_attacks_sp23.pdf), it's exactly 45ns.
/// I measured against UART at ~115200 baud, and got ~42ns, so I figure ~45ns is probably right if
/// we ignore overhead;
const RDTIME_TO_NANOS: u64 = 45;

fn now_raw() -> u64 {
    let mut out: u64;
    unsafe {
        asm!("rdtime {t}", t = out(reg) out);
    }
    out
}

#[derive(Debug, Copy, Clone)]
pub struct Instant(u64);

impl Instant {
    pub fn never() -> Instant {
        Instant(0)
    }
    pub fn now() -> Instant {
        Instant(now_raw())
    }
    pub fn from_raw(v: u64) -> Instant {
        Instant(v)
    }
}
impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        assert!(
            self.0 >= rhs.0,
            "Difference between `Instant`s cannot be negative"
        );
        let diff = self.0.wrapping_sub(rhs.0);
        Duration::from_nanos(diff * RDTIME_TO_NANOS)
    }
}

// Note: we don't care about rollover, since that would take about ~24ky

pub fn delay(millis: u64) {
    let start = Instant::now();
    while (Instant::now() - start) > Duration::from_millis(millis) {}
}
pub fn delay_micros(micros: u64) {
    let start = Instant::now();
    while (Instant::now() - start) > Duration::from_micros(micros) {}
}

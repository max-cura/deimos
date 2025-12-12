#![allow(dead_code)]

use core::{hint::unlikely, ops::Sub, time::Duration};

/// Get the counter frequency
pub fn cp15_cntfrq() -> u32 {
    let t: u32;
    unsafe {
        core::arch::asm!("mrc p15, 0, {t}, c14, c0, 0", t = out(reg) t);
    }
    t
}

pub fn read() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "mrrc p15, 0, {lo}, {hi}, c14",
            lo = out(reg) lo,
            hi = out(reg) hi,
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

fn floating_micros() -> u64 {
    read() / 24
}

#[derive(Debug, Copy, Clone)]
pub struct Instant(u64);
impl Instant {
    pub fn now() -> Self {
        Self(read())
    }
}
impl Sub<Instant> for Instant {
    type Output = core::time::Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let ticks = self.0 - rhs.0;
        // ticks is in units of 1/24 us
        // then ns = ticks * 1000/24
        let ns = ticks * 1000 / 24;
        Duration::from_nanos(ns)
    }
}

/// Blocking wait for (at least) `milliseconds` milliseconds.
/// Implemented on top of [`delay_micros`]; see that function's documentation for timing guarantees.
pub fn delay(mut milliseconds: u64) {
    const MAX_MILLIS_PER_STEP: u64 = u64::MAX / 1000;
    const SATURATE_TO_MICROS: u64 = MAX_MILLIS_PER_STEP * 1000;
    while milliseconds > MAX_MILLIS_PER_STEP {
        let microseconds = SATURATE_TO_MICROS;
        delay_micros(microseconds);
        milliseconds -= MAX_MILLIS_PER_STEP;
    }
    let microseconds = milliseconds * 1000;
    delay_micros(microseconds);
}

/// Blocking wait for (at least) `microseconds` microseconds. We can only make a guarantee that it
/// waits for at least `microseconds`, but in practice, in a no-interrupts setting, it should be
/// exact due to the difference in clock rate.
pub fn delay_micros(microseconds: u64) {
    let start = floating_micros();
    let end = start.wrapping_add(microseconds);
    if unlikely(end < start) {
        if unlikely(microseconds > (u32::MAX as u64)) {
            // wraparound: end < start <= u64::MAX
            const U64_HALF: u64 = u64::MAX / 2;
            // The first instinct is to just write:
            //  while now >= start || now < end {}
            // however, this breaks down for end == start-1 for micros=u64::MAX
            // so, we check whether we passed 0, and we can therefore use:
            //  while !passed_zero || now < end {}
            // however, this still breaks down for start=u64::MAX, micros=u64::MAX
            // so, we also check whether we passed u64::MAX/2 after passing zero; and once we pass
            // u64::MAX/2, if we go below u64::MAX/2 again, immediately stop the loop.
            let mut passed_zero = false;
            let mut passed_half = false;
            loop {
                let now = floating_micros();
                if now < start && !passed_zero {
                    passed_zero = true;
                }
                if passed_zero && now >= U64_HALF && !passed_half {
                    passed_half = true;
                }
                if (passed_zero && now >= end) || (passed_half && now < U64_HALF) {
                    break;
                }
            }
        } else {
            // small wraparound 0 <= end << start <= u64::MAX
            while {
                let now = floating_micros();
                now >= start || now < end
            } {
                // do nothing
            }
        }
    } else {
        // no wraparound: start <= end <= u64::MAX
        while {
            let now = floating_micros();
            start <= now && now < end
        } {
            // do nothing
        }
    }
}

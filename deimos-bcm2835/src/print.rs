use bcm2835_lpa::UART1;
use crate::arch::dsb;

pub struct UartProxy<'a> {
    inner: &'a UART1,
}

impl<'a> UartProxy<'a> {
    pub fn new(uart1: &'a UART1) -> Self {
        Self { inner: uart1 }
    }
    #[allow(unused)]
    pub fn flush(&mut self) {
        crate::uart::flush_tx_fifo(&self.inner);
    }
}

impl<'a> core::fmt::Write for UartProxy<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        dsb();
        for &b in s.as_bytes() {
            while !self.inner.stat().read().tx_ready().bit_is_set() {}
            self.inner.io().write(|w| unsafe { w.data().bits(b) });
        }
        dsb();
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
($($arg:tt)*) => {
    {
        #[allow(unused_imports)]
        use ::core::fmt::Write as _;
        let out = &unsafe { ::bcm2835_lpa::Peripherals::steal() }.UART1;
        let mut proxy = $crate::print::UartProxy::new(out);
        let _ = ::core::writeln!(proxy, $($arg)*);
        $crate::uart::flush_tx_fifo(out);
    }
}
}

#[macro_export]
macro_rules! print {
($($arg:tt)*) => {
    {
        #[allow(unused_imports)]
        use ::core::fmt::Write as _;
        let out = &unsafe { ::bcm2835_lpa::Peripherals::steal() }.UART1;
        let mut proxy = $crate::print::UartProxy::new(out);
        let _ = ::core::write!(proxy, $($arg)*);
        $crate::uart::flush_tx_fifo(out);
    }
}
}


#[macro_export]
// https://doc.rust-lang.org/stable/src/std/macros.rs.html#352-374
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}:{}]", ::core::file!(), ::core::line!(), ::core::column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}:{}] {} = {:#?}",
                    ::core::file!(), ::core::line!(), ::core::column!(), ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

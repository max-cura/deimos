#![feature(alloc_layout_extra)]
#![feature(decl_macro)]
#![no_std]

extern crate alloc;

pub mod dilf;
pub mod tests;

#[derive(Debug, Copy, Clone)]
pub struct Timing {
    pub cycle_begin: u32,
    pub cycle_end: u32,
}

impl Timing {
    pub fn cycles(&self) -> u32 {
        self.cycle_end.wrapping_sub(self.cycle_begin)
    }
}

pub macro set_impl($t:expr) {
    #[unsafe(no_mangle)]
    pub fn _sulfur_write(s: &str) {
        let _ = ::core::fmt::Write::write_str(&mut $t, s);
    }
    #[unsafe(no_mangle)]
    pub fn _sulfur_flush() {
        let mut t = $t;
        $crate::Flushable::flush(&mut t);
    }
}

unsafe extern "Rust" {
    pub fn _sulfur_write(s: &str);
    pub fn _sulfur_flush();
}

pub trait Flushable {
    fn flush(&mut self);
}

pub struct Printer;
impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { _sulfur_write(s) };
        Ok(())
    }
}
pub macro println($($arg:tt)*) {
    {
        use ::core::fmt::Write as _;
        let _ = ::core::writeln!($crate::Printer, $($arg)*);
        unsafe { $crate::_sulfur_flush() };
    }
}
pub macro print($($arg:tt)*) {
    {
        use ::core::fmt::Write as _;
        let _ = ::core::write!($crate::Printer, $($arg)*);
        unsafe { $crate::_sulfur_flush() };
    }
}
// https://doc.rust-lang.org/stable/src/std/macros.rs.html#352-374
pub macro dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}:{}]", ::core::file!(), ::core::line!(), ::core::column!())
    },
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
    },
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    },
}

macro_rules! dsb {
    ($name:ident) => {
        #[allow(dead_code)]
        pub fn $name() {
            unsafe { ::core::arch::asm!(::core::concat!("dsb ", ::core::stringify!($name))) }
        }
    };
}
macro_rules! dmb {
    ($name:ident) => {
        #[allow(dead_code)]
        pub fn $name() {
            unsafe { ::core::arch::asm!(::core::concat!("dmb ", ::core::stringify!($name))) }
        }
    };
}
pub mod dsb {
    dsb!(sy);
    dsb!(st);
    dsb!(ish);
    dsb!(ishst);
    dsb!(nsh);
    dsb!(nshst);
    dsb!(osh);
    dsb!(oshst);
}
pub mod dmb {
    dmb!(sy);
    dmb!(st);
    dmb!(ish);
    dmb!(ishst);
    dmb!(nsh);
    dmb!(nshst);
    dmb!(osh);
    dmb!(oshst);
}
pub mod isb {
    #[allow(dead_code)]
    pub fn sy() {
        unsafe { ::core::arch::asm!("isb sy") }
    }
}

#[allow(dead_code)]
pub fn dsb() {
    dsb::sy();
}
#[allow(dead_code)]
pub fn isb() {
    isb::sy();
}
#[allow(dead_code)]
pub fn dmb() {
    dmb::sy();
}

#![feature(decl_macro, likely_unlikely)]
#![no_std]
#![no_main]

extern crate alloc;

mod alloc_support;
mod arch;
mod cache;
mod ccu;
mod cp15;
mod critical_section;
mod print;
mod start;
mod timing;
mod uart;

use sulfur::println;

pub extern "C" fn kernel_start() {
    // LEDs on PA15 and PL10
    // let pa_cfg1 = 0x01c20804 as *mut u32;
    // let pa_dat = 0x01c20810 as *mut u32;
    // unsafe { pa_cfg1.write_volatile((pa_cfg1.read_volatile() & !0x7 << 28) | (0x1 << 28)); }
    // loop {
    //     for _ in 0..1000000 { unsafe { asm!("nop") } }
    //     unsafe { pa_dat.write_volatile(pa_dat.read_volatile() ^ 0x8000); }
    // }

    let _device = uart::Device::new_init(
        uart::RegisterBlock::from_addr(0x1c28000),
        24_000_000,
        115200,
    );

    println!("\x1b[32m--- PNEUMA ---\x1b[0m");

    alloc_support::heap_init();

    println!("\x1b[32m--- PNEUMA END ---\x1b[0m");

    start::reboot();
}

#[panic_handler]
pub fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = panic_info.location() {
        println!(
            "Panic occurred at file '{}' line {}:",
            loc.file(),
            loc.line()
        );
    } else {
        println!("Panic occurred at unknown location.");
    }
    let msg = panic_info.message();
    println!("{}", msg);

    start::reboot()
}

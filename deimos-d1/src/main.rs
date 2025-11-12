#![no_std]
#![no_main]
#![feature(alloc_layout_extra)]

extern crate alloc;

use sulfur::println;

core::arch::global_asm!(
    r#"
        .attribute arch, "rv64gc"
"#
);

mod alloc_support;
mod ccu;
mod critical_section;
mod dma;
mod ledc;
mod print;
mod start;
mod timing;
mod uart;

pub extern "C" fn kernel_start() -> ! {
    let peri = unsafe { d1_pac::Peripherals::steal() };

    ledc::init(&peri.GPIO, &peri.CCU);
    ledc::set(0xff_ff_ff);

    timing::delay(100);

    ccu::init(&peri.CCU);
    uart::init(&peri.CCU, &peri.GPIO, &peri.UART0, 200_000_000, 115200);

    // TODO: dcache, icache, brpdx, etc.

    alloc_support::heap_init();

    println!();
    println!("UART is up. {} booted.", env!("CARGO_BIN_NAME"));

    println!();

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

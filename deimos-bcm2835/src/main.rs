#![feature(likely_unlikely)]
#![feature(pointer_is_aligned_to)]
#![feature(slice_ptr_get)]
#![feature(alloc_layout_extra)]
#![feature(sync_unsafe_cell)]
#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

use bcm2835_lpa::Peripherals;

mod alloc_support;
mod arch;
mod coprocessor;
mod critical_section;
mod dma;
mod mailbox;
mod mmu_support;
mod print;
mod start;
mod timing;
mod uart;
mod watchdog;

#[unsafe(no_mangle)]
pub extern "C" fn __kernel_start() -> ! {
    let peri = unsafe { Peripherals::steal() };

    timing::delay_millis(&peri.SYSTMR, 100);

    uart::init(
        &peri.GPIO,
        &peri.AUX,
        &peri.UART1,
        uart::baud_to_clock_divider(115200),
    );

    unsafe extern "C" {
        static __exec_end: [u32; 0];
    }
    println!("executable area: 0..{:08x}", (&raw const __exec_end).addr());

    unsafe {
        mmu_support::init();
        mmu_support::set_mmu_enabled_features(mmu_support::MmuConfig {
            dcache: Some(false),
            icache: Some(false),
            brpdx: Some(false),
        });
    }

    alloc_support::heap_init();

    println!();
    println!("UART is up. {} booted.", env!("CARGO_BIN_NAME"));

    println!();
    mailbox::dump_configuration();

    println!();
    dma::run_all(&peri);

    watchdog::restart(&peri.PM);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let peri = unsafe { Peripherals::steal() };
    uart::init(
        &peri.GPIO,
        &peri.AUX,
        &peri.UART1,
        uart::baud_to_clock_divider(115200),
    );
    if let Some(loc) = info.location() {
        println!(
            "Panic occurred at file '{}' line {}:\n",
            loc.file(),
            loc.line()
        );
    } else {
        println!("Panic occurred at unknown location.\n");
    }
    let msg = info.message();
    let mut proxy = print::UartProxy::new(&peri.UART1);
    let _ = core::fmt::write(&mut proxy, format_args!("{}\n", msg));

    // wait for UART FIFO to drain
    timing::delay_millis(&peri.SYSTMR, 100);

    watchdog::restart(&peri.PM);
}

#[unsafe(no_mangle)]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}

//! Minimal benchmark helper for measuring Assura-built Rust CLI startup floor.

#![cfg_attr(all(unix, not(test)), no_main)]
#![cfg_attr(all(unix, not(debug_assertions), not(test)), no_std)]

#[cfg(all(unix, not(test)))]
use core::ffi::{c_char, c_int};
#[cfg(all(unix, not(debug_assertions), not(test)))]
use core::panic::PanicInfo;

#[cfg(not(unix))]
fn main() {
    std::process::exit(0);
}

#[cfg(all(target_vendor = "apple", unix, not(test)))]
#[link(name = "System")]
extern "C" {
    fn _exit(status: c_int) -> !;
}

#[cfg(all(not(target_vendor = "apple"), unix, not(test)))]
#[link(name = "c")]
extern "C" {
    fn _exit(status: c_int) -> !;
}

#[cfg(all(unix, not(test)))]
/// Raw Unix entrypoint that exits successfully without doing validation work.
///
/// # Safety
///
/// The platform C runtime must call this with the standard `argc`/`argv`
/// contract. The arguments are intentionally ignored.
#[no_mangle]
pub unsafe extern "C" fn main(_argc: c_int, _argv: *const *const c_char) -> c_int {
    _exit(0);
}

#[cfg(all(unix, not(debug_assertions), not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    unsafe {
        _exit(1);
    }
}

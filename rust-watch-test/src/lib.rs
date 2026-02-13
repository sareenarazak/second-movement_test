#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    unsafe fn watch_display_text(position: u8, message: *const u8);
}

#[unsafe(no_mangle)]
pub extern "C" fn test_watch_face() {
    let text = b"hello world\0";
    unsafe {
        watch_display_text(0, text.as_ptr());
    }
}

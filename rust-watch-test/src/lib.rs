#![no_std]
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    unsafe fn watch_display_string(message: *const u8, position: u8);
    unsafe fn watch_clear_display();
}
#[unsafe(no_mangle)]
pub extern "C" fn display_hello() {
    let text = b"hai";
    unsafe {
        watch_clear_display();
        watch_display_string(text.as_ptr(), 4);
    }
}

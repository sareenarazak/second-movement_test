#![no_std]
use core::ffi::c_char;
use core::fmt::Write;
use core::panic::PanicInfo;
use heapless::String;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const CLOCK_FACE_LOW_BATTERY_VOLTAGE_THRESHOLD: u16 = 2400;
#[repr(C)]
pub struct WatchDateTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub day: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ClockState {
    time_signal_enabled: bool,
    alarm_enabled: bool,
    battery_low: bool,
    last_battery_check: u8,
}

#[repr(C)]
pub enum WatchIndicator {
    Signal = 0,
    Bell,
    Pm,
    H24,
    Lap,
    Arrows,
    Sleep,
    Colon,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum WatchPosition {
    Full = 0,
    Top,
    TopLeft,
    TopRight,
    Bottom,
    Hours,
    Minutes,
    Seconds,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum MovementClockMode {
    Mode12h = 0,
    Mode24h,
    Mode024h,
    NumClockModes,
}

#[repr(C)]
pub enum WatchLcdType {
    Unknown = 0,
    Classic = 0b1010_1001,
    Custom = 0b0101_0110,
}

unsafe extern "C" {
    fn watch_set_indicator(indicator: WatchIndicator);
    fn watch_clear_indicator(indicator: WatchIndicator);
    fn movement_clock_mode_24h() -> MovementClockMode;
    fn movement_alarm_enabled() -> bool;
    fn watch_get_lcd_type() -> WatchLcdType;
    fn watch_get_vcc_voltage() -> u16;

    fn watch_display_text_with_fallback(
        position: WatchPosition,
        long: *const c_char,
        short: *const c_char,
    );

    fn watch_display_text(position: WatchPosition, text: *const c_char);
    fn watch_utility_get_weekday(date_time: &WatchDateTime) -> *const c_char;
    fn watch_utility_get_long_weekday(date_time: &WatchDateTime) -> *const c_char;
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_indicate(indicator: WatchIndicator, on: bool) {
    unsafe {
        if on {
            watch_set_indicator(indicator);
        } else {
            watch_clear_indicator(indicator);
        }
    }
}
// might change to take state input
#[unsafe(no_mangle)]
pub extern "C" fn clock_indicate_alarm() {
    unsafe {
        clock_indicate(WatchIndicator::Signal, movement_alarm_enabled());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_indicate_time_signal(state: *const ClockState) {
    unsafe {
        clock_indicate(WatchIndicator::Bell, (*state).time_signal_enabled);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_indicate_24h() {
    unsafe {
        clock_indicate(
            WatchIndicator::H24,
            movement_clock_mode_24h() == MovementClockMode::Mode12h,
        );
    }
}

pub fn clock_is_pm(date_time: WatchDateTime) -> bool {
    date_time.hour >= 12
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_indicate_pm(date_time: WatchDateTime) {
    unsafe {
        if movement_clock_mode_24h() != MovementClockMode::Mode24h {
            clock_indicate(WatchIndicator::Pm, clock_is_pm(date_time));
        }
    }
}
pub fn clock_indicate_low_available_power(state: ClockState) {
    unsafe {
        match watch_get_lcd_type() {
            WatchLcdType::Custom => {
                clock_indicate(WatchIndicator::Arrows, state.battery_low);
            }
            _ => {
                clock_indicate(WatchIndicator::Lap, state.battery_low);
            }
        }
    }
}
pub fn clock_24h_to_12h(mut date_time: WatchDateTime) -> WatchDateTime {
    date_time.hour %= 12;

    if date_time.hour == 0 {
        date_time.hour = 12;
    }
    date_time
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_check_battery_periodically(
    state: *mut ClockState,
    date_time: WatchDateTime,
) {
    unsafe {
        if date_time.day == (*state).last_battery_check {
            return;
        }
        (*state).last_battery_check = date_time.day;
        (*state).battery_low = watch_get_vcc_voltage() < CLOCK_FACE_LOW_BATTERY_VOLTAGE_THRESHOLD;
        clock_indicate_low_available_power(*state);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_toggle_time_signal(state: *mut ClockState) {
    unsafe {
        (*state).time_signal_enabled = !(*state).time_signal_enabled;
        clock_indicate_time_signal(state);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn clock_display_all(date_time: WatchDateTime) {
    unsafe {
        let mut buf: String<9> = String::new();

        if movement_clock_mode_24h() == MovementClockMode::Mode024h {
            write!(
                buf,
                "{:02}{:02}{:02}{:02}\0",
                date_time.day, date_time.hour, date_time.minute, date_time.second
            )
            .ok();
        } else {
            write!(
                buf,
                "{:2}{:2}{:02}{:02}\0",
                date_time.day, date_time.hour, date_time.minute, date_time.second
            )
            .ok();
        }
        watch_display_text_with_fallback(
            WatchPosition::TopLeft,
            watch_utility_get_long_weekday(&date_time),
            watch_utility_get_weekday(&date_time),
        );
        watch_display_text(WatchPosition::TopRight, buf.as_ptr() as *const c_char);
        watch_display_text(WatchPosition::Bottom, buf[2..].as_ptr() as *const c_char);
    }
}

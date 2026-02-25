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

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct RtcDateTime {
    pub reg: u32,
}

impl RtcDateTime {
    pub fn second(self) -> u8 {
        (self.reg & 0x3F) as u8
    }
    pub fn minute(self) -> u8 {
        ((self.reg >> 6) & 0x3F) as u8
    }
    pub fn hour(self) -> u8 {
        ((self.reg >> 12) & 0x1F) as u8
    }
    pub fn day(self) -> u8 {
        ((self.reg >> 17) & 0x1F) as u8
    }
    pub fn month(self) -> u8 {
        ((self.reg >> 22) & 0x0F) as u8
    }
    pub fn year(self) -> u8 {
        ((self.reg >> 26) & 0x3F) as u8
    }

    pub fn calendar_year(self) -> u16 {
        2020 + self.year() as u16
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WatchDateTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl From<RtcDateTime> for WatchDateTime {
    fn from(r: RtcDateTime) -> Self {
        Self {
            second: r.second(),
            minute: r.minute(),
            hour: r.hour(),
            day: r.day(),
            month: r.month(),
            year: r.year(),
        }
    }
}

impl From<WatchDateTime> for RtcDateTime {
    fn from(d: WatchDateTime) -> Self {
        Self {
            reg: (d.second as u32 & 0x3F)
                | ((d.minute as u32 & 0x3F) << 6)
                | ((d.hour as u32 & 0x1F) << 12)
                | ((d.day as u32 & 0x1F) << 17)
                | ((d.month as u32 & 0x0F) << 22)
                | ((d.year as u32 & 0x3F) << 26),
        }
    }
}
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
        text: *const c_char,
        fall_back: *const c_char,
    );

    fn watch_display_text(position: WatchPosition, text: *const c_char);
    fn watch_utility_get_weekday(date_time: RtcDateTime) -> *const c_char;
    fn watch_utility_get_long_weekday(date_time: RtcDateTime) -> *const c_char;
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
            movement_clock_mode_24h() == MovementClockMode::Mode24h,
        );
    }
}

pub fn clock_is_pm(date_time: WatchDateTime) -> bool {
    date_time.hour >= 12
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_indicate_pm(date_time: RtcDateTime) {
    unsafe {
        if movement_clock_mode_24h() == MovementClockMode::Mode24h {
            return;
        }
        let dt: WatchDateTime = date_time.into();
        clock_indicate(WatchIndicator::Pm, dt.hour >= 12);
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
pub extern "C" fn clock_check_battery_periodically(state: *mut ClockState, date_time: RtcDateTime) {
    unsafe {
        let dt: WatchDateTime = date_time.into();

        if dt.day == (*state).last_battery_check {
            return;
        }

        (*state).last_battery_check = dt.day;
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
pub extern "C" fn clock_display_all(date_time: RtcDateTime) {
    let dt: WatchDateTime = date_time.into();

    let mut date_time_s: String<8> = String::new();

    unsafe {
        if movement_clock_mode_24h() == MovementClockMode::Mode024h {
            write!(
                date_time_s,
                "{:02}{:02}{:02}{:02}",
                dt.day, dt.hour, dt.minute, dt.second
            )
            .ok();
        } else {
            write!(
                date_time_s,
                "{:2}{:2}{:02}{:02}",
                dt.day, dt.hour, dt.minute, dt.second
            )
            .ok();
        }

        let mut c_buf = [0u8; 9];

        let bytes = date_time_s.as_bytes();
        c_buf[..bytes.len()].copy_from_slice(bytes);

        watch_display_text_with_fallback(
            WatchPosition::TopLeft,
            watch_utility_get_long_weekday(date_time),
            watch_utility_get_weekday(date_time),
        );

        watch_display_text(WatchPosition::TopRight, c_buf.as_ptr() as *const c_char);

        watch_display_text(
            WatchPosition::Bottom,
            c_buf.as_ptr().add(2) as *const c_char,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_display_some(currentt: RtcDateTime, previous: RtcDateTime) {
    unsafe {}
}

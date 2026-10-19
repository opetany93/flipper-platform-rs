use core::ptr;

use flipperzero::notification::{NotificationApp, NotificationMessage, NotificationSequence};
use flipperzero::notification_sequence;
use flipperzero_sys as sys;

const BACKLIGHT_BRIGHTNESS_DEFAULT: u8 = 50;
const BACKLIGHT_BRIGHTNESS_STEP: u8 = 10;
const BACKLIGHT_BRIGHTNESS_MIN: u8 = BACKLIGHT_BRIGHTNESS_STEP;
const BACKLIGHT_BRIGHTNESS_MAX: u8 = 100;

/// Keeps the backlight on while alive; hold it in an `Option` to switch it at runtime.
///
/// The firmware counts backlight locks, so keep at most one alive at a time.
pub struct BacklightLock {
    /// `None` keeps the user's brightness from the firmware settings.
    level: Option<u8>,
}

impl BacklightLock {
    #[must_use]
    pub fn acquire() -> Self {
        const BACKLIGHT_ON: NotificationSequence =
            notification_sequence![NotificationMessage::display_backlight_enforce_on()];
        NotificationApp::open().notify_blocking(&BACKLIGHT_ON);

        Self { level: None }
    }

    #[must_use]
    pub fn with_brightness(percent: u8) -> Self {
        let level = percent.clamp(BACKLIGHT_BRIGHTNESS_MIN, BACKLIGHT_BRIGHTNESS_MAX);

        let brightness = brightness_setting(level);
        let enforce = NotificationMessage::display_backlight_enforce_on();
        notify_borrowed(&[
            ptr::from_ref(&brightness),
            ptr::from_ref(&enforce),
            ptr::null(),
        ]);

        Self { level: Some(level) }
    }

    pub fn increase(&mut self) {
        self.apply(
            self.current_level()
                .saturating_add(BACKLIGHT_BRIGHTNESS_STEP)
                .min(BACKLIGHT_BRIGHTNESS_MAX),
        );
    }

    pub fn decrease(&mut self) {
        self.apply(
            self.current_level()
                .saturating_sub(BACKLIGHT_BRIGHTNESS_STEP)
                .max(BACKLIGHT_BRIGHTNESS_MIN),
        );
    }

    /// Every key press makes the firmware show the user's brightness again, so call this after each input event.
    pub fn refresh(&self) {
        const REVEAL: NotificationSequence =
            notification_sequence![NotificationMessage::display_backlight(0)];
        NotificationApp::open().notify(&REVEAL);
    }

    // The user's brightness cannot be read back, so stepping from it starts at the default.
    fn current_level(&self) -> u8 {
        self.level.unwrap_or(BACKLIGHT_BRIGHTNESS_DEFAULT)
    }

    fn apply(&mut self, level: u8) {
        if Some(level) == self.level {
            return;
        }

        self.level = Some(level);

        // The firmware only takes the brightness of an enforced backlight when the lock is first taken.
        let brightness = brightness_setting(level);
        let release = NotificationMessage::display_backlight_enforce_auto();
        let enforce = NotificationMessage::display_backlight_enforce_on();
        let reveal = NotificationMessage::display_backlight(0);
        notify_borrowed(&[
            ptr::from_ref(&brightness),
            ptr::from_ref(&release),
            ptr::from_ref(&enforce),
            ptr::from_ref(&reveal),
            ptr::null(),
        ]);
    }
}

impl Drop for BacklightLock {
    fn drop(&mut self) {
        // Lighting the display afterwards restarts the firmware's backlight timeout from now.
        const BACKLIGHT_AUTO: NotificationSequence = notification_sequence![
            NotificationMessage::display_backlight_enforce_auto(),
            NotificationMessage::display_backlight(0xFF),
        ];
        NotificationApp::open().notify_blocking(&BACKLIGHT_AUTO);
    }
}

fn brightness_setting(percent: u8) -> NotificationMessage {
    NotificationMessage::force_display_brightness_setting(
        f32::from(percent) / f32::from(BACKLIGHT_BRIGHTNESS_MAX),
    )
}

// `notify` only accepts `'static` sequences; blocking keeps this one alive until it is processed.
fn notify_borrowed(sequence: &[*const NotificationMessage]) {
    assert!(sequence.last().is_some_and(|message| message.is_null()));
    unsafe {
        sys::notification_message_block(NotificationApp::open().as_ptr(), sequence.as_ptr().cast());
    }
}

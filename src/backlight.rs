use core::ptr;

use flipperzero::notification::{NotificationApp, NotificationMessage, NotificationSequence};
use flipperzero::notification_sequence;
use flipperzero_sys as sys;

const BACKLIGHT_BRIGHTNESS_DEFAULT: u8 = 50;
const BACKLIGHT_BRIGHTNESS_STEP: i8 = 10;
const BACKLIGHT_BRIGHTNESS_MIN: u8 = BACKLIGHT_BRIGHTNESS_STEP.unsigned_abs();
const BACKLIGHT_BRIGHTNESS_MAX: u8 = 100;

struct BacklightLock {
    /// `None` keeps the user's brightness from the firmware settings.
    percent: Option<u8>,
}

impl BacklightLock {
    #[must_use]
    fn acquire() -> Self {
        const BACKLIGHT_ON: NotificationSequence =
            notification_sequence![NotificationMessage::display_backlight_enforce_on()];
        NotificationApp::open().notify_blocking(&BACKLIGHT_ON);

        Self { percent: None }
    }

    #[must_use]
    fn with_brightness(percent: u8) -> Self {
        debug_assert!((BACKLIGHT_BRIGHTNESS_MIN..=BACKLIGHT_BRIGHTNESS_MAX).contains(&percent));
        let brightness = brightness_setting(percent);
        let enforce = NotificationMessage::display_backlight_enforce_on();
        notify_borrowed(&[
            ptr::from_ref(&brightness),
            ptr::from_ref(&enforce),
            ptr::null(),
        ]);

        Self {
            percent: Some(percent),
        }
    }

    fn increase(&mut self) {
        self.step_by(BACKLIGHT_BRIGHTNESS_STEP);
    }

    fn decrease(&mut self) {
        self.step_by(-BACKLIGHT_BRIGHTNESS_STEP);
    }

    fn refresh(&self) {
        const REVEAL: NotificationSequence =
            notification_sequence![NotificationMessage::display_backlight(0)];
        NotificationApp::open().notify(&REVEAL);
    }

    fn percent(&self) -> Option<u8> {
        self.percent
    }

    /// The user's brightness cannot be read back, so stepping from it starts at the default.
    fn step_by(&mut self, delta: i8) {
        let start = self.percent.unwrap_or(BACKLIGHT_BRIGHTNESS_DEFAULT);
        self.apply(
            start
                .saturating_add_signed(delta)
                .clamp(BACKLIGHT_BRIGHTNESS_MIN, BACKLIGHT_BRIGHTNESS_MAX),
        );
    }

    fn apply(&mut self, percent: u8) {
        if Some(percent) == self.percent {
            return;
        }

        self.percent = Some(percent);

        // The firmware only takes the brightness of an enforced backlight when the lock is first taken.
        let brightness = brightness_setting(percent);
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

/// The firmware counts backlight locks, so keep at most one alive at a time.
#[derive(Default)]
pub struct Backlight {
    lock: Option<BacklightLock>,
    percent: Option<u8>,
}

impl Backlight {
    #[must_use]
    pub fn with_brightness(percent: u8) -> Self {
        Self {
            lock: None,
            percent: Some(percent.clamp(BACKLIGHT_BRIGHTNESS_MIN, BACKLIGHT_BRIGHTNESS_MAX)),
        }
    }

    pub fn percent(&self) -> Option<u8> {
        self.lock
            .as_ref()
            .map_or(self.percent, BacklightLock::percent)
    }

    pub fn hold(&mut self) {
        self.lock.get_or_insert_with(|| {
            self.percent
                .map_or_else(BacklightLock::acquire, BacklightLock::with_brightness)
        });
    }

    pub fn release(&mut self) {
        if let Some(lock) = &self.lock {
            self.percent = lock.percent();
        }

        self.lock = None;
    }

    /// No-op while released
    pub fn increase(&mut self) {
        if let Some(lock) = &mut self.lock {
            lock.increase();
        }
    }

    /// No-op while released
    pub fn decrease(&mut self) {
        if let Some(lock) = &mut self.lock {
            lock.decrease();
        }
    }

    /// Every key press makes the firmware show the user's brightness again, so call this after each input event.
    pub fn refresh(&self) {
        if let Some(lock) = &self.lock {
            lock.refresh();
        }
    }
}

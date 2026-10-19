use core::ffi::CStr;
use core::marker::PhantomData;

use flipperzero_sys as sys;

static TOM_THUMB_4X6: &[u8] = include_bytes!("../fonts/tom_thumb_4x6_tr.bin");
static FIXED_5X7: &[u8] = include_bytes!("../fonts/5x7_tr.bin");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Smallest,
    Small,
    Regular,
    Bold,
    Keyboard,
    BigNumbers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Right,
    Center,
}

impl HorizontalAlign {
    const fn to_sys(self) -> sys::Align {
        match self {
            Self::Left => sys::AlignLeft,
            Self::Right => sys::AlignRight,
            Self::Center => sys::AlignCenter,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Bottom,
    Center,
}

impl VerticalAlign {
    const fn to_sys(self) -> sys::Align {
        match self {
            Self::Top => sys::AlignTop,
            Self::Bottom => sys::AlignBottom,
            Self::Center => sys::AlignCenter,
        }
    }
}

pub struct Canvas<'a> {
    raw: *mut sys::Canvas,
    _frame: PhantomData<&'a mut sys::Canvas>,
}

impl Canvas<'_> {
    /// # Safety
    ///
    /// `raw` must stay valid for the returned lifetime.
    pub(crate) unsafe fn from_raw(raw: *mut sys::Canvas) -> Self {
        Self {
            raw,
            _frame: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        unsafe { sys::canvas_clear(self.raw) };
    }

    pub fn set_font(&mut self, font: Font) {
        unsafe {
            match font {
                Font::Smallest => {
                    sys::canvas_set_custom_u8g2_font(self.raw, TOM_THUMB_4X6.as_ptr())
                }
                Font::Small => sys::canvas_set_custom_u8g2_font(self.raw, FIXED_5X7.as_ptr()),
                Font::Regular => sys::canvas_set_font(self.raw, sys::FontSecondary),
                Font::Bold => sys::canvas_set_font(self.raw, sys::FontPrimary),
                Font::Keyboard => sys::canvas_set_font(self.raw, sys::FontKeyboard),
                Font::BigNumbers => sys::canvas_set_font(self.raw, sys::FontBigNumbers),
            }
        };
    }

    /// `y` is the baseline, not the top edge.
    pub fn draw_str(&mut self, x: i32, y: i32, text: &CStr) {
        unsafe { sys::canvas_draw_str(self.raw, x, y, text.as_ptr()) };
    }

    pub fn draw_str_aligned(
        &mut self,
        x: i32,
        y: i32,
        horizontal: HorizontalAlign,
        vertical: VerticalAlign,
        text: &CStr,
    ) {
        unsafe {
            sys::canvas_draw_str_aligned(
                self.raw,
                x,
                y,
                horizontal.to_sys(),
                vertical.to_sys(),
                text.as_ptr(),
            )
        };
    }

    pub fn string_width(&mut self, text: &CStr) -> u16 {
        unsafe { sys::canvas_string_width(self.raw, text.as_ptr()) }
    }

    pub fn draw_xbm(&mut self, x: i32, y: i32, width: usize, height: usize, bitmap: &[u8]) {
        let required = width
            .div_ceil(8)
            .checked_mul(height)
            .expect("bitmap dimensions overflow");

        assert!(
            bitmap.len() >= required,
            "bitmap too short for {width}x{height}"
        );

        unsafe { sys::canvas_draw_xbm(self.raw, x, y, width, height, bitmap.as_ptr()) };
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        unsafe {
            sys::canvas_draw_line(self.raw, x1, y1, x2, y2);
        };
    }
}

# flipper-platform

Safe Rust wrappers over the Flipper Zero firmware, built on the
[`flipperzero`](https://crates.io/crates/flipperzero) crate, with borrowed
callbacks instead of boxed `'static` ones.

- **Borrowed callbacks.** View port and timer callbacks are plain closures
  borrowed for the wrapper's lifetime: no boxing, no `'static`. A view port only
  exists inside `ViewPort::scoped`, so its callbacks are always detached before
  anything they capture goes away.
- **RAII everywhere.** Every wrapper releases what it acquired on drop.
- **No `unsafe` at the call site.**

## Modules

| Module      | What it wraps                                              |
| ----------- | ---------------------------------------------------------- |
| `view_port` | A fullscreen view port with draw and input callbacks       |
| `canvas`    | Text, lines and XBM bitmaps on the canvas a draw call gets |
| `input`     | Key events as exhaustively matchable enums                 |
| `timer`     | Periodic Furi timers                                       |
| `backlight` | Keeping the display backlight on while a guard is alive    |
| `battery`   | Battery charge level and charging state                    |

## Example

```rust,ignore
use flipper_platform::backlight::BacklightLock;
use flipper_platform::canvas::{Canvas, Font};
use flipper_platform::input::InputEvent;
use flipper_platform::view_port::ViewPort;

let _backlight = BacklightLock::acquire();

let on_draw = |canvas: &mut Canvas<'_>| {
    canvas.clear();
    canvas.set_font(Font::Bold);
    canvas.draw_str(2, 12, c"Hello, Flipper");
};
let on_input = |_event: InputEvent| {};

ViewPort::scoped(&on_draw, &on_input, |view_port| {
    // The app's event loop goes here.
    view_port.request_redraw();
});
```

The view port is removed from the GUI when `scoped` returns, and dropping
`_backlight` hands the backlight back to the firmware.

## Status

Early: the API covers what its first application needed and grows from there.

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/opetany93/flipper-platform-rs/blob/master/LICENSE-APACHE)
or [MIT license](https://github.com/opetany93/flipper-platform-rs/blob/master/LICENSE-MIT)
at your option.

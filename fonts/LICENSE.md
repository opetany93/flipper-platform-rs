# Fonts

Extracted from [u8g2](https://github.com/olikraus/u8g2) `csrc/u8g2_fonts.c` with
`scripts/extract_u8g2_font.py`. Each file is the raw u8g2 font array, including
the trailing NUL of the C string literal.

The u8g2 library code is BSD-2-Clause, but its fonts carry their own licenses:

| File                   | Font                                                      | License                        |
| ---------------------- | --------------------------------------------------------- | ------------------------------ |
| `tom_thumb_4x6_tr.bin` | Tom Thumb by Brian J. Swetland, modified by Robey Pointer | CC0 ([details][tom-thumb])     |
| `5x7_tr.bin`           | X11 Misc-Fixed 7px                                        | Public domain ([details][x11]) |

[tom-thumb]: https://github.com/olikraus/u8g2/wiki/fntgrptomthumb
[x11]: https://github.com/olikraus/u8g2/wiki/fntgrpx11

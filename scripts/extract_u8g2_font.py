"""Extract fonts from upstream u8g2 `csrc/u8g2_fonts.c` into raw `.bin` files."""

import argparse
import re
import sys
from pathlib import Path

FONT_PREFIX = "u8g2_font_"
STRING_LITERAL = re.compile(r'"((?:[^"\\]|\\.)*)"')
ESCAPE = re.compile(r"\\([0-7]{1,3}|x[0-9a-fA-F]+|.)")
SIMPLE_ESCAPES = {
    "a": 0x07, "b": 0x08, "f": 0x0C, "n": 0x0A, "r": 0x0D,
    "t": 0x09, "v": 0x0B, "\\": 0x5C, "'": 0x27, '"': 0x22, "?": 0x3F,
}


def unescape(literal: str) -> bytes:
    out = bytearray()
    position = 0
    for match in ESCAPE.finditer(literal):
        out += literal[position:match.start()].encode("latin-1")
        out.append(escape_value(match.group(1)))
        position = match.end()
    out += literal[position:].encode("latin-1")
    return bytes(out)


def escape_value(escape: str) -> int:
    if escape[0] in "01234567":
        return int(escape, 8)
    if escape[0] == "x":
        return int(escape[1:], 16)
    return SIMPLE_ESCAPES[escape]


def extract(source: str, name: str) -> bytes:
    escaped_name = re.escape(name)
    declaration = re.compile(
        rf'const uint8_t {escaped_name}\[(\d+)\] U8G2_FONT_SECTION\("{escaped_name}"\) ='
        rf'((?:\s*{STRING_LITERAL.pattern})+)\s*;'
    )
    match = declaration.search(source)
    if match is None:
        raise ValueError(f"font not found: {name}")

    declared_size = int(match.group(1))
    # The C array also holds the literal's implicit terminating NUL.
    data = b"".join(unescape(s) for s in STRING_LITERAL.findall(match.group(2))) + b"\0"
    if len(data) != declared_size:
        raise ValueError(f"{name}: decoded {len(data)} bytes, declared {declared_size}")
    return data


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="path to u8g2_fonts.c")
    parser.add_argument("out_dir", type=Path)
    parser.add_argument("fonts", nargs="+", help="e.g. u8g2_font_4x6_tr")
    args = parser.parse_args()

    source = args.source.read_text(encoding="latin-1")
    args.out_dir.mkdir(parents=True, exist_ok=True)
    for name in args.fonts:
        target = args.out_dir / f"{name.removeprefix(FONT_PREFIX)}.bin"
        try:
            data = extract(source, name)
        except ValueError as error:
            sys.exit(str(error))
        target.write_bytes(data)
        print(f"{name} -> {target}")


if __name__ == "__main__":
    main()

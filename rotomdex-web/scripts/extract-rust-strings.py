#!/usr/bin/env python3

"""Write the unique printable symbols used by Rust string literals."""

from pathlib import Path
import sys
from typing import List, Optional, Tuple


class ScanError(ValueError):
    pass


def _raw_string(source: str, start: int, prefix_length: int) -> Optional[Tuple[str, int]]:
    cursor = start + prefix_length
    while cursor < len(source) and source[cursor] == "#":
        cursor += 1
    if cursor >= len(source) or source[cursor] != '"':
        return None

    hashes = source[start + prefix_length : cursor]
    terminator = '"' + hashes
    value_start = cursor + 1
    value_end = source.find(terminator, value_start)
    if value_end == -1:
        raise ScanError("unterminated raw string literal")
    return source[value_start:value_end], value_end + len(terminator)


def _escape(source: str, start: int) -> tuple[str, int]:
    if start + 1 >= len(source):
        raise ScanError("unterminated escape sequence")

    escaped = source[start + 1]
    simple_escapes = {
        "0": "\0",
        "t": "\t",
        "n": "\n",
        "r": "\r",
        '"': '"',
        "'": "'",
        "\\": "\\",
    }
    if escaped in simple_escapes:
        return simple_escapes[escaped], start + 2

    if escaped == "x":
        digits = source[start + 2 : start + 4]
        if len(digits) != 2 or any(character not in "0123456789abcdefABCDEF" for character in digits):
            raise ScanError("invalid ASCII escape")
        return chr(int(digits, 16)), start + 4

    if escaped == "u" and source.startswith("{", start + 2):
        end = source.find("}", start + 3)
        if end == -1:
            raise ScanError("unterminated Unicode escape")
        digits = source[start + 3 : end].replace("_", "")
        if not digits or any(character not in "0123456789abcdefABCDEF" for character in digits):
            raise ScanError("invalid Unicode escape")
        try:
            return chr(int(digits, 16)), end + 1
        except ValueError as error:
            raise ScanError("invalid Unicode code point") from error

    if escaped in "\n\r":
        cursor = start + 2
        if escaped == "\r" and cursor < len(source) and source[cursor] == "\n":
            cursor += 1
        while cursor < len(source) and source[cursor].isspace():
            cursor += 1
        return "", cursor

    raise ScanError(f"unsupported escape sequence \\{escaped}")


def _cooked_string(source: str, start: int) -> tuple[str, int]:
    value = []
    cursor = start + 1
    while cursor < len(source):
        character = source[cursor]
        if character == '"':
            return "".join(value), cursor + 1
        if character == "\\":
            escaped, cursor = _escape(source, cursor)
            value.append(escaped)
        else:
            value.append(character)
            cursor += 1
    raise ScanError("unterminated string literal")


def _character_end(source: str, start: int) -> Optional[int]:
    cursor = start + 1
    if cursor >= len(source) or source[cursor] in "'\n\r":
        return None
    if source[cursor] == "\\":
        try:
            _, cursor = _escape(source, cursor)
        except ScanError:
            return None
    else:
        cursor += 1
    return cursor + 1 if cursor < len(source) and source[cursor] == "'" else None


def extract_strings(source: str) -> List[str]:
    strings = []
    cursor = 0
    while cursor < len(source):
        if source.startswith("//", cursor):
            newline = source.find("\n", cursor + 2)
            cursor = len(source) if newline == -1 else newline + 1
            continue

        if source.startswith("/*", cursor):
            depth = 1
            cursor += 2
            while depth and cursor < len(source):
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            if depth:
                raise ScanError("unterminated block comment")
            continue

        skipped = None
        for prefix in ("br", "cr"):
            if source.startswith(prefix, cursor):
                skipped = _raw_string(source, cursor, len(prefix))
                if skipped is not None:
                    break
        if skipped is not None:
            _, cursor = skipped
            continue

        if source.startswith(('b"', 'c"'), cursor):
            _, cursor = _cooked_string(source, cursor + 1)
            continue

        if source.startswith("b'", cursor):
            end = _character_end(source, cursor + 1)
            if end is not None:
                cursor = end
                continue

        raw = _raw_string(source, cursor, 1) if source[cursor] == "r" else None
        if raw is not None:
            value, cursor = raw
            strings.append(value)
            continue

        if source[cursor] == '"':
            value, cursor = _cooked_string(source, cursor)
            strings.append(value)
            continue

        if source[cursor] == "'":
            end = _character_end(source, cursor)
            if end is not None:
                cursor = end
                continue

        cursor += 1

    return strings


def main(paths: List[str]) -> int:
    symbols = set()
    for path_string in paths:
        path = Path(path_string)
        try:
            strings = extract_strings(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, ScanError) as error:
            print(f"error: unable to scan '{path}': {error}", file=sys.stderr)
            return 1
        symbols.update(character for value in strings for character in value if character.isprintable())
    print("".join(sorted(symbols)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

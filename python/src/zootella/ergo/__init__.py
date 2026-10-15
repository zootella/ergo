"""ergo, short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.

# region size
>>> from zootella.ergo import Size, say_size4
>>> buffer = bytearray(64 * Size.KB)  # a buffer of 64 kibibytes, rather than a bare 65536
>>> say_size4(123456789)  # four digits or less, with a unit
'117 MB'

# endregion size

# region time
>>> from zootella.ergo import Time
>>> 1700000000000 + 2 * Time.HOUR  # two hours after a tick, which counts milliseconds since 1970
1700007200000

# endregion time

# region tick
>>> from zootella.ergo import tick_to_text, text_to_tick, tick_to_iso, iso_to_tick
>>> t = 1344230277300  # a tick: milliseconds since 1970; this one is the Curiosity landing
>>> tick_to_text(t)  # as short as the moment allows
'2012aug6.0517.57300'
>>> tick_to_iso(t)  # always 24 characters
'2012-08-06T05:17:57.300Z'
>>> text_to_tick("2012aug6.0517.57300") == t == iso_to_tick("2012-08-06T05:17:57.300Z")  # each form reads back exactly
True

# endregion tick

# region tag
>>> from zootella.ergo import tag, has_tag, check_tag
>>> t = tag()  # a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals from the operating system's random source
>>> has_tag(t)
True
>>> has_tag("not a tag")
False
>>> check_tag(t) == t  # returns t, or raises because the caller handed it something that isn't a tag
True

# endregion tag
"""

import math
import re
import secrets
from datetime import UTC, datetime, timedelta

# size


class Size:
    """Numbers of bytes in the binary units that memory and storage are measured in, 1024 to each rather than 1000, so 64 * Size.KB names what a bare 65536 would hide."""

    B = 1  # one byte, the unit
    KB = 1024 * B  # kibibyte
    MB = 1024 * KB  # mebibyte
    GB = 1024 * MB  # gibibyte
    TB = 1024 * GB  # tebibyte
    PB = 1024 * TB  # pebibyte, the last; 2^50 is still a safe integer everywhere


# each 1024 of the one before, labeled the everyday way; bytes come before these, and are the one unit that pluralizes
_size_units = [" KB", " MB", " GB", " TB", " PB", " EB", " ZB", " YB"]


def say_size4(n: int) -> str:
    """Say a count of bytes in four digits or less with a unit, like "1 byte", "2 bytes", or "117 MB"; n is zero or more, and the count rounds down."""
    if n <= 9999:  # at most four digits of bytes; singular only for exactly one
        return f"{n} byte{'' if n == 1 else 's'}"
    w = n  # whole units of the unit we're counting in
    for unit in _size_units:
        w //= Size.KB  # count in the next unit up
        if w <= 9999:
            return f"{w}{unit}"  # fits in four digits, so this is the unit to say it in
    return str(n)  # past yottabytes, which only an unbounded int reaches; say it bare


# time


class Time:
    """Spans of time in milliseconds, the unit of a tick, so now + 2 * Time.HOUR is a deadline two hours out."""

    MILLISECOND = 1  # the unit; a tick counts milliseconds since 1970
    SECOND = 1000 * MILLISECOND
    MINUTE = 60 * SECOND
    HOUR = 60 * MINUTE
    DAY = 24 * HOUR
    WEEK = 7 * DAY
    # a year is 365 and a quarter days, the average over the leap cycle, and a month is a twelfth of that; both are exact in milliseconds
    YEAR = 365 * DAY + DAY // 4
    MONTH = YEAR // 12


# tick

# A tick is an integer count of milliseconds since the start of 1970, and ergo keeps every tick between zero and the last millisecond of 9999. Two text forms carry a tick exactly. The iso form is like "2012-08-06T05:17:57.300Z": always 24 characters, always milliseconds, always Z. The short form is like "2012aug6.0517.57300": the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs, so the start of 1990 is just "1990" and the longest text is 20 characters.
#
# Each form reads back to exactly the tick that wrote it, and reads nothing else. A reader accepts the text its writer produces and refuses everything else, from a missing millisecond digit to "1990feb1" where the writer would say "1990feb". The last check in each reader is the round trip itself: parse the text, write the tick, and require the same text. That's what keeps every text one to one with its tick.

_last_tick = 253402300799999  # the last millisecond of 9999, which keeps a year at four digits in every edition
_epoch = datetime(1970, 1, 1, tzinfo=UTC)  # tick zero
_months = [
    "jan",
    "feb",
    "mar",
    "apr",
    "may",
    "jun",
    "jul",
    "aug",
    "sep",
    "oct",
    "nov",
    "dec",
]
_iso_pattern = re.compile(
    r"([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})\.([0-9]{3})Z"
)  # the one shape of the iso form; [0-9] rather than \d, which would admit other scripts' digits
_numerals_pattern = re.compile(r"[0-9]+")


def _split_tick(t: int) -> datetime:
    """The utc moment a tick names, or raise because the caller handed us something that isn't a tick."""
    if not isinstance(t, int) or t < 0 or t > _last_tick:
        raise ValueError(f"not a tick: {t!r}")
    return _epoch + timedelta(
        milliseconds=t
    )  # exact, since timedelta keeps whole milliseconds as integers


def _join_tick(y: int, m: int, d: int, h: int, i: int, ms: int) -> int | None:
    """The tick at the given utc parts, with ms the milliseconds into the minute, or None if there's no such moment, like february 30, or it's outside the range of ticks."""
    if y < 1970 or not (
        1 <= m <= 12 and 1 <= d <= 31 and h <= 23 and i <= 59 and ms <= 59999
    ):
        return None
    try:
        moment = datetime(y, m, d, h, i, tzinfo=UTC)
    except ValueError:  # the platform speaks exceptions for a day that doesn't exist; we answer with a value
        return None
    t = (moment - _epoch) // timedelta(milliseconds=1) + ms
    return t if t <= _last_tick else None


def tick_to_iso(t: int) -> str:
    """Write a tick in the iso form, like "2012-08-06T05:17:57.300Z"."""
    d = _split_tick(t)
    return f"{d.year:04}-{d.month:02}-{d.day:02}T{d.hour:02}:{d.minute:02}:{d.second:02}.{d.microsecond // 1000:03}Z"


def iso_to_tick(s: str) -> int:
    """Read the iso form back to its tick, or raise because the text isn't exactly what a tick writes."""
    m = _iso_pattern.fullmatch(
        s
    )  # fullmatch rather than match with $, which would forgive a trailing newline
    t = (
        _join_tick(
            *(int(g) for g in m.groups()[:5]), int(m.group(6)) * 1000 + int(m.group(7))
        )
        if m
        else None
    )
    if t is None or tick_to_iso(t) != s:  # the round trip is the final check
        raise ValueError(f"not the iso form of a tick: {s!r}")
    return t


def tick_to_text(t: int) -> str:
    """Write a tick in the short form, like "2012aug6.0517.57300", as deep as the moment needs and no deeper."""
    d = _split_tick(t)
    ms = (
        d.second * 1000 + d.microsecond // 1000
    )  # milliseconds into the minute, 0 through 59999
    depth = 0  # how deep the text must go: the year alone, then the month, day, hour, minute, and millisecond
    if d.month > 1:
        depth = 1
    if d.day > 1:
        depth = 2
    if d.hour > 0:
        depth = 3
    if d.minute > 0:
        depth = 4
    if ms > 0:
        depth = 5
    text = str(d.year)
    if depth >= 1:
        text += _months[d.month - 1]
    if depth >= 2:
        text += str(d.day)
    if depth >= 3:
        text += f".{d.hour:02}"
    if depth >= 4:
        text += f"{d.minute:02}"
    if depth >= 5:
        text += f".{ms}"
    return text


def text_to_tick(s: str) -> int:
    """Read the short form back to its tick, or raise because the text isn't exactly what a tick writes."""
    t = _parse_text(s)
    if (
        t is None or tick_to_text(t) != s
    ):  # the round trip is the final check, catching february 30 and a padded or deeper text alike
        raise ValueError(f"not the short form of a tick: {s!r}")
    return t


def _parse_text(s: str) -> int | None:
    """Read the short form leniently enough to compute a tick, or None; text_to_tick above makes it strict with the round trip."""
    parts = s.split(
        "."
    )  # the year, month, and day; then the clock; then the milliseconds into the minute
    if len(parts) > 3:
        return None
    p1 = parts[0]
    if len(p1) not in (4, 7, 8, 9) or not _numerals_pattern.fullmatch(
        p1[:4]
    ):  # "1990", "1990feb", "1990feb2", "1990feb28"
        return None
    y, m, d, h, i, ms = int(p1[:4]), 1, 1, 0, 0, 0
    if len(p1) >= 7:
        if p1[4:7] not in _months:
            return None
        m = _months.index(p1[4:7]) + 1
    if len(p1) >= 8:
        if not _numerals_pattern.fullmatch(p1[7:]):
            return None
        d = int(p1[7:])
    if len(parts) >= 2:
        p2 = parts[1]
        if len(p2) not in (2, 4) or not _numerals_pattern.fullmatch(
            p2
        ):  # "09" an hour, "0905" an hour and minute
            return None
        h = int(p2[:2])
        if len(p2) == 4:
            i = int(p2[2:])
    if len(parts) >= 3:
        if not _numerals_pattern.fullmatch(parts[2]):
            return None
        ms = int(parts[2])
    return _join_tick(y, m, d, h, i, ms)


# tag

TAG_LENGTH = (
    21  # long enough to be unique, short enough to be reasonable, and nanoid's default
)

# letters and numerals only, so a double-click selects the whole tag; nanoid's default alphabet also has - and _
_tag_alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
_tag_pattern = re.compile(r"[0-9A-Za-z]+")
_tag_mask = 63  # the alphabet has 62 characters; 63 is the smallest all-ones mask that covers them
# random bytes per draw; 1.6 is nanoid's margin so one draw almost always suffices
_tag_step = math.ceil(1.6 * _tag_mask * TAG_LENGTH / len(_tag_alphabet))


def tag() -> str:
    """Generate a new universally unique tag of 21 letters and numerals, like "AgKxru95C7jFp5iPuK9O7"."""
    # the algorithm is nanoid's: mask random bytes down to the alphabet's size, and skip the few values that land past its end, so every character is equally likely
    tag = ""
    while True:
        # one call to the operating system per draw, rather than one per character, which was 14 times slower
        for b in secrets.token_bytes(_tag_step):
            i = b & _tag_mask  # 0 through 63
            if i < len(_tag_alphabet):
                tag += _tag_alphabet[
                    i
                ]  # 62 and 63 fall past the end; skipping them keeps the distribution uniform
            if len(tag) == TAG_LENGTH:
                return tag


def has_tag(s: object) -> bool:
    """True if s is exactly 21 letters and numerals."""
    # fullmatch rather than match with $, which would accept a trailing newline
    return (
        isinstance(s, str)
        and len(s) == TAG_LENGTH
        and _tag_pattern.fullmatch(s) is not None
    )


def check_tag(s: str) -> str:
    """A guard: pass a tag through, or raise because the caller handed us something that isn't one."""
    if not has_tag(s):
        raise ValueError(f"not a tag: {s!r}")
    return s

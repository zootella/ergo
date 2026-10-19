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
>>> from zootella.ergo import tick_to_text, text_to_tick, tick_to_iso, iso_to_tick, has_tick
>>> t = 1344230277300  # a tick: milliseconds since 1970; this one is the Curiosity landing
>>> tick_to_text(t)  # as short as the moment allows
'2012aug6.0517.57300'
>>> tick_to_iso(t)  # always 24 characters
'2012-08-06T05:17:57.300Z'
>>> text_to_tick("2012aug6.0517.57300") == t == iso_to_tick("2012-08-06T05:17:57.300Z")  # each form reads back exactly
True
>>> has_tick(-1)  # ask about a number from outside, like a clock or a file's modified time, before handing it to a function that takes a tick
False

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

# region hash
>>> from zootella.ergo import BLAKE3, SHA1, SHA256, Size, sha256
>>> sha256(b"hello").hex()  # all at once, for bytes already in memory: 32 bytes
'2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
>>> import io
>>> file = io.BytesIO(b"hello")  # or open(path, "rb"), a file of any size
>>> hashers = [SHA1(), SHA256(), BLAKE3()]  # or a piece at a time, all three from one read of the file
>>> while piece := file.read(64 * Size.KB):
...     for h in hashers:
...         h.update(piece)
>>> for h in hashers:
...     print(h.digest().hex())
aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f

# endregion hash
"""

import hashlib
import math
import re
import secrets
from collections.abc import Buffer
from datetime import UTC, datetime, timedelta
from typing import Protocol

from blake3 import blake3 as _blake3

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
#
# Every function that takes a tick raises on a number that isn't one, because a caller handing it one has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so has_tick asks about it first, and asking never raises; once has_tick says yes, nothing here raises on that tick. check_tick is the guard that enforces the same answer, and it's the gate every function here goes through.

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


def has_tick(t: object) -> bool:
    """True if t is a tick, an integer from zero through the last millisecond of 9999; never raises, so a number from outside can be asked about first."""
    # a bool is an int in python, and never a tick
    return isinstance(t, int) and not isinstance(t, bool) and 0 <= t <= _last_tick


def check_tick(t: int) -> int:
    """A guard: pass a tick through, or raise because the caller handed us something that isn't one."""
    if not has_tick(t):
        raise ValueError(f"not a tick: {t!r}")
    return t


def _split_tick(t: int) -> datetime:
    """The utc moment a tick names, or raise because the caller handed us something that isn't a tick."""
    # exact, since timedelta keeps whole milliseconds as integers
    return _epoch + timedelta(milliseconds=check_tick(t))


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


# hash

# Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.
#
# Each hash comes in two forms that give the same answer. The function takes all the bytes at once, like sha256(data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with SHA256(), call update with each piece in order, and call digest for the hash. A file of any size hashes this way in the memory of one piece, and a single read of the file can feed all three hashers. Digest doesn't finish a hasher, as in hashlib: update it again and digest again for the hash of everything so far.
#
# hashlib has SHA-1 and SHA-256, on OpenSSL or verified code of its own, and no BLAKE3, which CPython declined to add, so BLAKE3 comes from the blake3 package, the official Python binding to the official Rust implementation. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.


def sha1(data: Buffer) -> bytes:
    """The SHA-1 hash of data, 20 bytes; for matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions."""
    return hashlib.sha1(data).digest()


def sha256(data: Buffer) -> bytes:
    """The SHA-256 hash of data, 32 bytes."""
    return hashlib.sha256(data).digest()


def blake3(data: Buffer) -> bytes:
    """The BLAKE3 hash of data, 32 bytes."""
    return _blake3(data).digest()


class _HashObject(Protocol):
    """What hashlib's hashers and the blake3 package's have in common, and all the hashers below need of them."""

    def update(self, data: Buffer, /) -> None: ...
    def digest(self) -> bytes: ...


class _Hasher:
    """The shape the three hashers below share: update with each piece in order, then digest for the hash of them all."""

    def __init__(self, h: _HashObject) -> None:
        self._h = h

    def update(self, data: Buffer) -> None:
        """Hash the next piece."""
        self._h.update(data)

    def digest(self) -> bytes:
        """The hash of every piece so far."""
        return self._h.digest()


class SHA1(_Hasher):
    """A SHA-1 hasher, a piece at a time, for the same 20 bytes sha1 gives all at once."""

    def __init__(self) -> None:
        super().__init__(hashlib.sha1())


class SHA256(_Hasher):
    """A SHA-256 hasher, the same way, for 32 bytes."""

    def __init__(self) -> None:
        super().__init__(hashlib.sha256())


class BLAKE3(_Hasher):
    """A BLAKE3 hasher, the same way, for 32 bytes."""

    def __init__(self) -> None:
        super().__init__(_blake3())

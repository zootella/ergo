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

# region data
>>> from zootella.ergo import Bin, Data
>>> d = Data(text="hello")  # bytes, from text
>>> d.base16(), d.base32(), d.base62(), d.base64()  # out in any form
('68656c6c6f', 'NBSWY3DP', 'Q6LhR6z', 'aGVsbG8=')
>>> Data(base62="Q6LhR6z").text()  # and back from any, each read exactly
'hello'
>>> d.clip_view(1, 3).text()  # part of it, without copying
'el'
>>> b = Bin(6)  # a set number of bytes, filled a byte or a Data at a time
>>> b.add(d)
>>> b.add(0x21)
>>> b.data().text()
'hello!'

# endregion data

# region hash
>>> from zootella.ergo import BLAKE3, SHA1, SHA256, Data, Size, sha256
>>> sha256(Data(text="hello")).base16()  # all at once, for bytes already in memory: a Data of 32 bytes, out in any form
'2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
>>> import io
>>> file = io.BytesIO(b"hello")  # or open(path, "rb"), a file of any size
>>> hashers = [SHA1(), SHA256(), BLAKE3()]  # or a piece at a time, all three from one read of the file
>>> while piece := file.read(64 * Size.KB):
...     for h in hashers:
...         h.update(Data(array=piece))
>>> for h in hashers:
...     print(h.digest().base16())
aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f

# endregion hash

# region fuji
>>> from zootella.ergo import hash_file, hash_stream
>>> import io
>>> tip = hash_file(io.BytesIO(b"hello"), 5).tip_hash  # or open(path, "rb") and its size; the tip hash, from a few small reads
>>> piece = hash_stream(io.BytesIO(b"hello"), 5).piece_hash  # the piece hash, from one read of all of it, which gives the tip hash too
>>> tip.base32(), piece.base32()
('BSOEFHWYKUFE2ZEYFGKAE2X4IZXTXXCDJZQ2YRGKLMAETUJDTIHQ', 'AD4G5U6L4LJC4DYUIUSFRIYHH5KMRVHCLOQQAKAPNNPFCRAUIV3Q')

# endregion fuji
"""

import base64
import binascii
import hashlib
import math
import re
import secrets
import time
import unicodedata
from collections.abc import Buffer, Callable
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import BinaryIO, NamedTuple, Protocol

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


# data

# A Data holds bytes. Every language keeps bytes its own way, in arrays and buffers and views with their own rules about copying and size, and every platform call that takes or gives bytes wants its own one of them. A Data makes those choices once: it holds the bytes, and it speaks every form they travel in, text and base16, base32, base62, and base64, through methods with plain names. Make one from any form, read it out as any other, and it's the same bytes. In Python a Data holds a memoryview, so it can look at bytes, a bytearray, or part of either without copying them.
#
# Every function in ergo that trades in bytes trades in Data. A hash takes a Data and answers one, and never decides what text the caller wants next; the caller says, with the next method. Only Data knows the encodings.
#
# Each form is exact, both ways. Every reader ends by writing the bytes back out and requiring the text it read, so it accepts only what its writer produces: base16 in lowercase, base32 without padding, base64 with it, nothing extra and nothing missing. Text becomes UTF-8 in Unicode Normalization Form C, so the same characters typed two ways become the same bytes, and reading bytes as text refuses bytes that aren't UTF-8, or aren't in that form, rather than guess.
#
# clip_view looks at part of a Data without copying it, which is fast, and right whenever the bytes beneath won't change. clip_copy copies them out, for when they could. array hands over the bytes as Python's own immutable bytes, which copies them unless the Data already holds exactly a bytes object.


class Data:
    """Bytes, made from exactly one form, like Data(text="hello") or Data(base16="68656c6c6f"), and read out as any: array, text, base16, base32, base62, or base64; random=n makes n bytes from the operating system's random source."""

    __slots__ = ("_base16", "_base32", "_base62", "_base64", "_text", "_view")

    def __init__(
        self,
        *,
        array: Buffer | None = None,
        text: str | None = None,
        base16: str | None = None,
        base32: str | None = None,
        base62: str | None = None,
        base64: str | None = None,
        random: int | None = None,
    ) -> None:
        given = [
            k
            for k, v in {
                "array": array,
                "text": text,
                "base16": base16,
                "base32": base32,
                "base62": base62,
                "base64": base64,
                "random": random,
            }.items()
            if v is not None
        ]
        if len(given) != 1:
            raise TypeError(
                f"Data takes exactly one of array, text, base16, base32, base62, base64, or random, not {given}"
            )
        # a Data always holds a view, keeps the given form, and makes and keeps the others as asked
        self._text = self._base16 = self._base32 = self._base62 = self._base64 = None
        if array is not None:
            # look at the given bytes without copying them
            self._view = memoryview(array).cast("B")
        elif text is not None:
            # keep the text in the form its bytes hold, so reading it back gives the same text
            self._text = _normal_text(text)
            self._view = memoryview(self._text.encode("utf-8"))
        elif base16 is not None:
            self._view, self._base16 = memoryview(_base16_to_bytes(base16)), base16
        elif base32 is not None:
            self._view, self._base32 = memoryview(_base32_to_bytes(base32)), base32
        elif base62 is not None:
            self._view, self._base62 = memoryview(_base62_to_bytes(base62)), base62
        elif base64 is not None:
            self._view, self._base64 = memoryview(_base64_to_bytes(base64)), base64
        else:
            self._view = memoryview(secrets.token_bytes(_check_count(random)))

    def size(self) -> int:
        """The size in bytes."""
        return len(self._view)

    def array(self) -> bytes:
        """The bytes, as Python's own bytes, copied unless this Data holds exactly a bytes object already."""
        whole = self._view.obj
        if isinstance(whole, bytes) and len(whole) == len(self._view):
            return whole
        return self._view.tobytes()

    def text(self) -> str:
        """The bytes as text, or raise because they aren't UTF-8 in Normalization Form C."""
        if self._text is None:
            self._text = _bytes_to_text(self._view)
        return self._text

    def base16(self) -> str:
        """The bytes in base16, two lowercase digits each."""
        if self._base16 is None:
            self._base16 = self._view.hex()
        return self._base16

    def base32(self) -> str:
        """The bytes in base32, uppercase and without padding."""
        if self._base32 is None:
            self._base32 = _bytes_to_base32(self._view)
        return self._base32

    def base62(self) -> str:
        """The bytes in base62."""
        if self._base62 is None:
            self._base62 = _bytes_to_base62(self._view)
        return self._base62

    def base64(self) -> str:
        """The bytes in base64, with its padding."""
        if self._base64 is None:
            self._base64 = _bytes_to_base64(self._view)
        return self._base64

    def get(self, i: int) -> int:
        """The byte at index i, a number 0x00 0 through 0xff 255, or raise because there's no byte there."""
        # a negative index would read from the end in python, and nowhere else
        if not (
            isinstance(i, int) and not isinstance(i, bool) and 0 <= i < len(self._view)
        ):
            raise IndexError(f"no byte {i!r} in {len(self._view)} bytes")
        return self._view[i]

    def clip_view(self, start: int, end: int) -> "Data":
        """The bytes from index start up to end, looked at without copying; fast, and right when the bytes beneath won't change."""
        _check_clip(len(self._view), start, end)
        return Data(array=self._view[start:end])

    def clip_copy(self, start: int, end: int) -> "Data":
        """The bytes from index start up to end, copied out, for when the bytes beneath could change 🪏."""
        _check_clip(len(self._view), start, end)
        return Data(array=self._view[start:end].tobytes())

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Data) and self._view == other._view

    __hash__ = None  # a Data can look at bytes that change, so it isn't a key

    def __repr__(self) -> str:
        return f"Data(base16={self.base16()!r})"


def _check_data(d: object) -> Data:
    """A guard: pass a Data through, or raise because the caller handed us something else."""
    if not isinstance(d, Data):
        raise TypeError(f"expected a Data: {d!r}")
    return d


def _check_count(n: object) -> int:
    """A guard: pass a count through, zero or more within the integer ceiling, or raise; a bool is an int in python, and never a count."""
    if not (isinstance(n, int) and not isinstance(n, bool) and 0 <= n <= 2**53 - 1):
        raise ValueError(f"not a count: {n!r}")
    return n


def _check_clip(size: int, start: object, end: object) -> None:
    """Start through end are byte indices from the start, so a clip of nothing is fine, but not one past the end, or backwards."""
    if not (
        isinstance(start, int)
        and isinstance(end, int)
        and not isinstance(start, bool)
        and not isinstance(end, bool)
        and 0 <= start <= end <= size
    ):
        raise IndexError(f"not a clip of {size} bytes: {start!r} to {end!r}")


def _round_trip(form: str, s: str, a: bytes, write) -> bytes:
    """The last check in every reader: write the bytes back out, and require exactly the text it read."""
    if write(a) != s:
        raise ValueError(f"not {form}: {s!r}")
    return a


def _normal_text(s: str) -> str:
    """Text in unicode Normalization Form C, so accents hash and match consistently however they were typed, or raise because it holds a lone surrogate, half of a pair, which has no UTF-8 form."""
    t = unicodedata.normalize("NFC", s)
    try:
        t.encode("utf-8")
    except UnicodeEncodeError:
        # the platform speaks an exception for text it can't encode; we raise our own, with the text
        raise ValueError(f"not text: {s!r}") from None
    return t


def _bytes_to_text(a: Buffer) -> str:
    """Bytes as text, strictly: UTF-8, which keeps a leading byte order mark as the character it is, and in Normalization Form C, since other bytes would become different bytes as text."""
    try:
        s = bytes(a).decode("utf-8")
    except UnicodeDecodeError:
        raise ValueError(f"not text: {bytes(a).hex()}") from None
    if not unicodedata.is_normalized("NFC", s):
        raise ValueError(f"not text in Normalization Form C: {bytes(a).hex()}")
    return s


def _base16_to_bytes(s: str) -> bytes:
    """Bytes from base16, strictly; fromhex alone would also take uppercase and skip spaces, both of which the round trip refuses."""
    try:
        a = bytes.fromhex(s)
    except ValueError:
        raise ValueError(f"not base16: {s!r}") from None
    return _round_trip("base16", s, a, bytes.hex)


def _bytes_to_base64(a: Buffer) -> str:
    return base64.b64encode(a).decode("ascii")


def _base64_to_bytes(s: str) -> bytes:
    """Bytes from base64, strictly; validate refuses characters outside the alphabet, and the round trip refuses missing padding and a last character with bits that belong to no byte."""
    try:
        a = base64.b64decode(s, validate=True)
    except binascii.Error:
        raise ValueError(f"not base64: {s!r}") from None
    return _round_trip("base64", s, a, _bytes_to_base64)


# base32, for hash values in the database, so they can be short, but also always the same length


def _bytes_to_base32(a: Buffer) -> str:
    # without the padding, as icarus writes it
    return base64.b32encode(a).decode("ascii").rstrip("=")


def _base32_to_bytes(s: str) -> bytes:
    """Bytes from base32, strictly; the standard library's reader wants padding, so add it, and the round trip refuses padding given, lowercase, and a last character with bits that belong to no byte."""
    try:
        a = base64.b32decode(s + "=" * (-len(s) % 8))
    except binascii.Error:
        raise ValueError(f"not base32: {s!r}") from None
    return _round_trip("base32", s, a, _bytes_to_base32)


# base62
#
# Why so many different encoding formats? base16 is good because it's simple, standard, and you can see the bytes. base32 is shorter, double-clickable, and always the same length, good for storing hash values in the database. base62 is shorter and double-clickable, good for bytes in the query string of api calls and the location bar the user might see. base64 is about the same length, standard, but not double-clickable, included here for completeness.

# narrow characters in popular spots, for data
_alphabet62_stream = "1023456789ABCDEFIGHJKLMNOPQRSTUVjWXYZabcdefghkmnlopqrstuvwxyzi"
#                     ^               ^               ^               ^            ^
#                     1               I               j               l            i  moved characters
#                     0               16              32              48           61  to these indices
#
# Encoding random short data, some characters appear more frequently than others.
# The last character, at index 61, is the most common.
# Also, the characters at 0, 16, 32, and 48 are slightly more common.
# Rendered to pixels for a user, we want base62 text to be as short as possible!
# So, we've adjusted the alphabet to put narrow characters in the popular spots.


def _bytes_to_base62(a: Buffer) -> str:
    """Turn data into text using base 62, each 4 or 6 bits will become a character 0-9, a-z, and A-Z."""
    a = bytes(a)
    i = 0  # the index in bits, from 0 through all the bits in the given data
    s = []  # target characters to build up and join
    while i < len(a) * 8:  # when the bit index moves beyond the memory, we're done
        # the same index as a distance in bytes followed by a distance in bits
        byte_index = i // 8
        bit_index = i % 8

        # copy the two bytes at byte_index into pair; on the very last byte, leave the right byte in pair all 0s
        pair = a[byte_index] << 8
        if byte_index + 1 < len(a):
            pair |= a[byte_index + 1]

        # read the 6 bits at i as a number, called code, which will be 0 through 63
        code = (pair >> (10 - bit_index)) & 63

        # describe the 6 bits with a numeral or letter, 111100 is 60, if 61 move forward 4, not 6
        if code < 61:
            # 000000 0 through 111100 60 append character at that index
            s.append(_alphabet62_stream[code])
            i += 6
        else:
            # 111101 61, 111110 62, and 111111 63 append 'i' and move past '1111'
            s.append(_alphabet62_stream[61])
            i += 4
    return "".join(s)


def _base62_to_bytes(s: str) -> bytes:
    """Bytes from base62, strictly; the round trip refuses a last character with bits that belong to no byte."""
    # more characters become fewer bytes, so this capacity will always be plenty
    b = Bin(len(s))
    # a place to hold bits from several characters until we have 8 and can write a byte
    hold = 0
    bits = 0  # the number of bits stored in the right side of hold right now
    for c in s:
        code = _alphabet62_stream.find(c)
        if code < 0:
            raise ValueError(f"not base62: {s!r}")

        # insert the bits from code into hold
        if code == 61:
            # insert 1111 for common 'i' at last alphabet index 61
            hold = (hold << 4) | 15
            bits += 4
        else:
            # insert 000000 through 111100 for other characters
            hold = (hold << 6) | code
            bits += 6

        # if we have enough bits in hold to write a byte
        if bits >= 8:
            # move the 8 leftmost bits in hold to our Bin object
            b.add((hold >> (bits - 8)) & 0xFF)
            # remove the bits we wrote from hold, any extra bits there will be written next time
            bits -= 8
            # and forget them, since python's ints never overflow the way javascript's do
            hold &= (1 << bits) - 1
    return _round_trip("base62", s, b.array(), _bytes_to_base62)


# bin


class Bin:
    """A set number of bytes, which you add a byte or a Data at a time until it's full; it's how to assemble bytes whose size you know before you start, and refusing to hold more than that catches a mistake in the prediction."""

    __slots__ = ("_array", "_size")

    def __init__(self, capacity: int) -> None:
        self._array = bytearray(_check_count(capacity))  # how many bytes it can hold
        self._size = 0  # how many bytes it does hold

    def capacity(self) -> int:
        """How many bytes it can hold."""
        return len(self._array)

    def size(self) -> int:
        """How many bytes it does hold."""
        return self._size

    def add(self, p: int | Data) -> None:
        """Add a byte, 0 through 255, or all of a Data, or raise because it won't fit."""
        if isinstance(p, int) and not isinstance(p, bool):
            if not 0 <= p <= 255:
                raise ValueError(f"not a byte: {p}")
            if self._size + 1 > len(self._array):
                raise ValueError(f"a Bin of {len(self._array)} bytes is full")
            self._array[self._size] = p
            self._size += 1
        elif isinstance(p, Data):
            n = p.size()
            if self._size + n > len(self._array):
                raise ValueError(
                    f"a Bin of {len(self._array)} bytes holding {self._size} can't take {n} more"
                )
            self._array[self._size : self._size + n] = p._view
            self._size += n
        else:
            raise TypeError(f"a Bin takes a byte or a Data: {p!r}")

    def array(self) -> bytes:
        """What it holds so far, copied out as bytes."""
        return bytes(self._array[: self._size])

    def data(self) -> Data:
        """What it holds so far, as a Data that looks at it without copying."""
        return Data(array=memoryview(self._array)[: self._size])


# hash

# Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.
#
# Each hash takes a Data and answers one, and comes in two forms that give the same answer. The function takes all the bytes at once, like sha256(data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with SHA256(), call update with each piece in order, and call digest for the hash. A file of any size hashes this way in the memory of one piece, and a single read of the file can feed all three hashers. Digest doesn't finish a hasher, as in hashlib: update it again and digest again for the hash of everything so far.
#
# hashlib has SHA-1 and SHA-256, on OpenSSL or verified code of its own, and no BLAKE3, which CPython declined to add, so BLAKE3 comes from the blake3 package, the official Python binding to the official Rust implementation. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.


def sha1(data: Data) -> Data:
    """The SHA-1 hash of a Data, as a Data of 20 bytes; for matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions."""
    return Data(array=hashlib.sha1(_check_data(data)._view).digest())


def sha256(data: Data) -> Data:
    """The SHA-256 hash of a Data, as a Data of 32 bytes."""
    return Data(array=hashlib.sha256(_check_data(data)._view).digest())


def blake3(data: Data) -> Data:
    """The BLAKE3 hash of a Data, as a Data of 32 bytes."""
    return Data(array=_blake3(_check_data(data)._view).digest())


def hash_text(s: str) -> str:
    """The SHA-256 hash of text, as base32, the shortcut for the common case; text to Data, hashed, and out as the form icarus keeps hash values in."""
    return sha256(Data(text=s)).base32()


class _HashObject(Protocol):
    """What hashlib's hashers and the blake3 package's have in common, and all the hashers below need of them."""

    def update(self, data: Buffer, /) -> None: ...
    def digest(self) -> bytes: ...


class _Hasher:
    """The shape the three hashers below share: update with each piece in order, then digest for the hash of them all."""

    def __init__(self, h: _HashObject) -> None:
        self._h = h

    def update(self, data: Data) -> None:
        """Hash the next piece, a Data."""
        self._h.update(_check_data(data)._view)

    def digest(self) -> Data:
        """The hash of every piece so far, as a Data."""
        return Data(array=self._h.digest())


class SHA1(_Hasher):
    """A SHA-1 hasher, a piece at a time, each a Data, for the same Data of 20 bytes sha1 gives all at once."""

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


# hash measure

# The fuji hashes are two of ergo's own, from icarus, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.
#
# The pieces are there by design, for JavaScript, whose Web Crypto hashes only all at once: hashing 4 MiB pieces and then the list of them lets a page hash a file of any size at Web Crypto's full speed. Here hashlib streams anyway, and each piece fills a bytearray of its own, which a Data then looks at without a copy, where JavaScript slides a conveyor belt. The tip hash can come two ways that must agree: from a file, by seeking to just the stripes, or from a stream, by noticing the stripes as they pass on the way to the piece hash.


class HashMeasurement(NamedTuple):
    """The pieces of a file at a unit, and the stripes the tip hash reads, each a start and end measured from the start of the file."""

    pieces: int  # total number of pieces
    complete: int  # quantity of complete pieces
    fragment: int  # byte size of last piece if it is a fragment
    stripes: list[tuple[int, int]]
    stripe_size: int  # how many bytes all the stripes hold


def hash_measure(file: int, unit: int) -> HashMeasurement:
    """Given a file size and a unit, measure the pieces of that size, and the stripes the tip hash reads 📏; or raise because both the file size and piece size must be 1 or more bytes."""
    if not (
        isinstance(file, int) and isinstance(unit, int) and file >= 1 and unit >= 1
    ):
        raise ValueError(f"not a file and unit to measure: {file!r}, {unit!r}")
    pieces = -(-file // unit)
    complete = file // unit
    fragment = file % unit
    if file <= 4 * unit:
        # file size is 4 units or less, so just hash the whole thing
        return HashMeasurement(pieces, complete, fragment, [(0, file)], file)
    first = 0
    middle = file // (2 * unit) * unit  # byte index of middle piece
    penultimate = (pieces - 2) * unit  # second to last piece, must always be complete
    last_size = fragment if fragment > 0 else unit  # how big the last stripe is
    stripes = [(first, first + unit)]  # first and middle never touch
    if middle + unit == penultimate:
        # but middle and penultimate can, and penultimate and last always do
        stripes.append((middle, middle + unit + unit + last_size))
    else:
        stripes.append((middle, middle + unit))  # middle with space before and after
        stripes.append((penultimate, penultimate + unit + last_size))
    return HashMeasurement(pieces, complete, fragment, stripes, 3 * unit + last_size)


def hash_stripe_overlap(
    foreground: tuple[int, int], background: list[tuple[int, int]]
) -> list[tuple[int, int]]:
    """Given a foreground stripe and a background pattern of stripes that don't touch or overlap, the ranges where they overlap, an empty list for none."""
    a, b = foreground
    return [(max(a, c), min(b, d)) for c, d in background if a < d and b > c]


# hash file and stream


class HashUnit(NamedTuple):
    """One of a protocol's two units: the title its summary starts with, and the size of a piece or a stripe."""

    title: str
    size: int


class HashProtocol(NamedTuple):
    """The units of the piece hash and the tip hash."""

    pieces: HashUnit
    tips: HashUnit


# a new simple protocol that can hash huge files on both the server and the page; introducing the "Fuji" system 🗻
HASH_PROTOCOL = HashProtocol(
    pieces=HashUnit("Fuji.Pieces.SHA256.4MiB.", 4 * Size.MB),
    tips=HashUnit("Fuji.Tips.SHA256.4KiB.", 4 * Size.KB),
)
# 4 MiB hashes in ~10ms similar to the frequency of page animation frames,
# uploads in under a second over a typical cable modem, and
# keeps the index of hash values smaller than the piece size for files under 500 GB 🗄️
# 4 KiB matches the cluster and sector size on Windows NTFS and the block size on Mac APFS,
# so hard drive reads can be as fast as possible 💽


@dataclass
class HashStatus:
    """What hash_file and hash_stream know so far, given to a progress callback as it changes and returned at the end."""

    total_size: int
    hashed_size: int = 0
    tip_hash: Data | None = None
    piece_hash: Data | None = None
    start_time: int = 0  # a tick, milliseconds since 1970
    update_time: int = 0  # when we last changed anything here
    duration: int | None = None  # milliseconds, once it's done


def _now() -> int:
    """The tick now."""
    return time.time_ns() // 1_000_000


def _full(bin: "Bin") -> Data:
    """The Data a Bin holds, or raise because it isn't exactly full, which means we predicted wrong how much data we'd hash."""
    if bin.size() != bin.capacity():
        raise ValueError(
            f"a Bin of {bin.capacity()} bytes holding {bin.size()} isn't full"
        )
    return bin.data()


def hash_file(
    file: BinaryIO, size: int, protocol: HashProtocol = HASH_PROTOCOL
) -> HashStatus:
    """Given a binary file that can seek, open to read, and its size, quickly compute just the tip hash, reading only the stripes."""
    # based on the file size, pick stripes at the start, middle, and end for us to hash quickly
    measure = hash_measure(size, protocol.tips.size)
    status = HashStatus(total_size=size, start_time=_now(), update_time=_now())

    # for tip hashing, the summary we'll hash is the title followed by stripes of file data (hashing file data, not hashes); different sized files hash differently even with identical tips
    title = Data(text=f"{protocol.tips.title}{size}.")
    tips = Bin(title.size() + measure.stripe_size)
    tips.add(title)
    for start, end in measure.stripes:
        file.seek(start)
        tips.add(Data(array=_read_exactly(file, end - start)))

    # hash the summary of the file in the bin, which must be exactly full, a check on our prediction of how much data we'd hash
    status.tip_hash = sha256(_full(tips))
    # we actually hashed more because of the title, but don't count this as to speed
    status.hashed_size = measure.stripe_size
    status.update_time = _now()
    status.duration = status.update_time - status.start_time
    return status


def _read_exactly(file: BinaryIO, n: int) -> bytes:
    """Exactly n bytes from file, or raise because it ended first; a read can return fewer than it was asked for."""
    a = bytearray()
    while len(a) < n:
        b = file.read(n - len(a))
        if not b:
            raise ValueError(f"the file ended {n - len(a)} bytes short of a stripe")
        a += b
    return bytes(a)


def hash_stream(
    stream: BinaryIO,
    size: int,
    protocol: HashProtocol = HASH_PROTOCOL,
    on_progress: Callable[[HashStatus], None] | None = None,
) -> HashStatus:
    """Given a binary stream to read and its size, compute both the tip hash and the piece hash in one pass."""
    # measurements for the piece hash, and for the tip hash, which we'll peek for through the stream
    measure_pieces = hash_measure(size, protocol.pieces.size)
    measure_tips = hash_measure(size, protocol.tips.size)
    status = HashStatus(total_size=size, start_time=_now(), update_time=_now())

    # for the pieces hash, the summary we'll hash is the title followed by hashes of pieces of file data, each SHA-256 hash 32 bytes (hashing hashes, not file data)
    pieces_title = Data(text=f"{protocol.pieces.title}{size}.")
    pieces = Bin(pieces_title.size() + 32 * measure_pieces.pieces)
    pieces.add(pieces_title)

    # for the tips hash, the summary we'll hash is the title followed by the start, middle, and end stripes of data
    tips_title = Data(text=f"{protocol.tips.title}{size}.")
    tips = Bin(tips_title.size() + measure_tips.stripe_size)
    tips.add(tips_title)

    def hash_piece(piece: bytearray) -> None:
        """Hash a piece and add its hash to the summary."""
        pieces.add(sha256(Data(array=piece)))
        status.hashed_size += len(piece)
        status.update_time = _now()
        if on_progress:
            on_progress(status)

    # the piece we're filling, which a Data will look at without a copy once it's full
    piece = bytearray()
    address = 0  # how far into the stream each box that arrives is
    while box_bytes := stream.read(64 * Size.KB):
        # a new delivery of bytes from the stream has arrived! 📦
        box = Data(array=box_bytes)
        # where in the stream this box of data is; then, outside the box, move address past it to mark the start of the next one
        stripe = (address, address + box.size())
        address += box.size()
        if address > size:
            raise ValueError(
                f"the stream is longer than the {size} bytes it was said to be"
            )

        # first, for the tip hash, we don't have random access through the file, but we do through this box!
        for start, end in hash_stripe_overlap(stripe, measure_tips.stripes):
            # subtract box address to clip in data
            tips.add(box.clip_view(start - stripe[0], end - stripe[0]))

        # second, for the piece hash, shovel bytes from the box into the piece until the box is empty 🪏
        shoveled = 0  # we move this index over the bytes in the box as we process them
        while shoveled < box.size():
            shovel = min(box.size() - shoveled, protocol.pieces.size - len(piece))
            piece += box.clip_view(shoveled, shoveled + shovel)._view
            shoveled += shovel
            if len(piece) == protocol.pieces.size:
                hash_piece(piece)
                piece = bytearray()

    # most files will end with a fragment piece smaller than a full piece
    if piece:
        hash_piece(piece)
    if status.hashed_size != size:
        raise ValueError(
            f"the stream was {status.hashed_size} bytes, not the {size} it was said to be"
        )

    # hash the summaries of the file in the bins
    status.piece_hash = sha256(_full(pieces))
    status.tip_hash = sha256(_full(tips))
    status.update_time = _now()
    status.duration = status.update_time - status.start_time
    if on_progress:
        on_progress(status)
    return status

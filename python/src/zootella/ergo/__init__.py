"""ergo, short for ergonomic: simple helper functions, the same in JavaScript, Python, Rust, and Go.

# region int
>>> from zootella.ergo import has_int, check_int
>>> has_int(9007199254740991)  # 2^53 - 1, the largest integer every edition holds exactly
True
>>> has_int(2**53)  # past the ceiling, where a double can't tell one integer from the next
False
>>> check_int(1344230277300)  # passes an integer through, or raises because the caller handed it something that isn't one
1344230277300

# endregion int

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

# region sayduration
>>> from zootella.ergo import Time, say_duration, say_time_race, say_time_remaining
>>> say_duration(2 * Time.HOUR + 5 * Time.MINUTE, "# hours # minutes")  # the template is the sentence, with # where each number goes
'2 hours 5 minutes'
>>> say_duration(3 * Time.DAY, "about # days ago")
'about 3 days ago'
>>> say_time_remaining(9 * Time.MINUTE + 59 * Time.SECOND)  # one or two units, whichever matter at the size
'9m 59s'
>>> print(say_time_race(5 * Time.MINUTE + 15 * Time.SECOND + 223))  # to the millisecond, the way a race is timed
5'15"223

# endregion sayduration

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

# region saytick
>>> from zootella.ergo import say_tick
>>> t = 1344230277300  # the Curiosity landing
>>> say_tick(t, "Dddd, Mmmm D, YYYY")  # uppercase tags are the calendar
'Monday, August 6, 2012'
>>> say_tick(t, "YYYY-MM-DD hh24:mm:ss.ttt")  # lowercase tags are the clock
'2012-08-06 05:17:57.300'
>>> say_tick(t, "h12:mm AA")
'5:17 AM'

# endregion saytick

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
>>> from zootella.ergo import FujiHasher, Data
>>> hasher = FujiHasher(5)  # or feed one the pieces as they arrive, like the chunks of an upload
>>> hasher.update(Data(text="he"))
>>> hasher.update(Data(text="llo"))
>>> hasher.digest() == (tip, piece)  # both hashes, the same as from the stream
True

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

# int

# An integer in ergo is one every edition holds exactly: a whole number from -(2^53 - 1) through 2^53 - 1, JavaScript's safe range. Past it, a double can't tell one integer from the next, so a number that crossed from Rust or Go into a page could quietly change. has_int asks whether a value is one, and asking never raises, so a number from outside can be asked about first; check_int is the guard that enforces the same answer.

# the largest integer a double holds along with every integer below it
_int_ceiling = 2**53 - 1


def has_int(i: object) -> bool:
    """True if i is an integer every edition holds exactly, a whole number from -(2^53 - 1) through 2^53 - 1; never raises, so a number from outside can be asked about first."""
    # a bool is an int in python, and never an integer here
    return (
        isinstance(i, int)
        and not isinstance(i, bool)
        and -_int_ceiling <= i <= _int_ceiling
    )


def check_int(i: int) -> int:
    """A guard: pass an integer through, or raise because the caller handed us something that isn't one."""
    if not has_int(i):
        raise ValueError(f"not an integer: {i!r}")
    return i


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
    """Say a count of bytes in four digits or less with a unit, like "1 byte", "2 bytes", or "117 MB". The count rounds down.

    n is a count, a whole number zero or more within the integer ceiling, and anything else raises ValueError, so ask has_int(n) and n >= 0 first about a number from outside.
    """
    # a fraction of a byte, or a negative count, is a bug in the caller, which should have made it whole
    _check_count(n)
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


# say_duration says a span of time for a person to read: how long something took, or how long ago it happened, like "2 hours 5 minutes" or "about 3 days ago". The template is the sentence it writes, with a # wherever a number goes. A tag is that #, any whitespace, and the unit after it: years, months, weeks, days, hours, minutes, or seconds, or one of the short forms yrs, yr, y, mos, mo, wks, w, d, hrs, hr, h, mins, min, m, secs, sec, and s, each written in lowercase, title case, or all capitals, like days, Days, and DAYS. Seconds can also take a fraction, #.#, #.##, or #.###, for tenths, hundredths, or thousandths, and no other unit can. Each # after the point is a digit always written, so 1.5 seconds says "1.500" under #.###, the way Excel's 0.000 writes it rather than its #.###. The whitespace can be any character Unicode calls white space: a space, a tab, or the no-break space a typographer puts there to keep "5 days" on one line.
#
# The units a template names split the span between them, and a template lists them from largest to smallest. The largest takes everything above it, so a year said in days is 365, and each smaller unit takes what's left. Everything rounds down, so whatever is smaller than the smallest unit goes unsaid. A year and a month are Time.YEAR and Time.MONTH, the averages over the leap cycle, so say_duration agrees with the rest of ergo, and a span of 365 days is a quarter of a day short of a year.
#
# A bare m is the one word that could be two units. On its own it's minutes, the way "5m" is in a feed, but because a template runs from largest to smallest, an m right after years, or right before weeks, days, or hours, can only be months, so "#y #m #d" says "1y 2m 3d" and "#d #h #m" says "1d 2h 3m".
#
# A plural, whole or short, comes out singular for exactly 1, in the case it was written in, so "# days" says "1 day" and "2 days", "# hrs" says "1 hr", and "# DAYS" says "1 DAY"; a single letter, like d, never changes.
#
# Units that come out zero at the start drop away, along with the text after them, so a template ready to say days can count up a task half an hour old: "#d #h #m #.##s" says "32m 55.45s", never "0d 0h 32m 55.45s". From the first unit that isn't zero, everything stays. A zero between two others stays, so a day and a second is "1 day 0 hours 0 minutes 1 second" rather than "1 day 1 second". And the smallest unit a template names is the precision it asks for, so it always shows, even on an exact amount: "#m #s" says "1m 0s" for exactly a minute. If every unit is zero, the last and smallest one says so, like "0 minutes".
#
# A unit only counts right after a #, so words and numbers around the tags pass through as they are: "about # days ago" says "about 3 days ago", and the d in "and" is only a letter. Only those exact spellings count, so a mix like hOURS is the tag h and then text, the way #hiring is. Whatever isn't a tag is written as it is, with no escaping and nothing refused, so a broken template writes text that's only wrong. The span is what must be right: a number of milliseconds within the integer ceiling, and anything else raises. A fraction of a millisecond, like the one time.time() * 1000 carries, drops away toward zero, so the sign stays with the size; nothing reads this text back, so there's no exactness to keep. That means the say functions take a number like 1.5 that has_int refuses, since an integer to carry has no fraction; to ask first about a span that might have one, ask has_int about math.trunc of it. A negative span says its size with a minus sign in front of everything, like "-2 hours 5 minutes", unless what it says is zero.

# each unit's words, whole and short, to the size of the unit in milliseconds, which is also how the code tells units apart; each longer word comes before its prefixes, since the first that matches wins, and a template writes each in lowercase, title case, or all capitals
_duration_units = {
    "years": Time.YEAR,
    "yrs": Time.YEAR,
    "yr": Time.YEAR,
    "y": Time.YEAR,
    "months": Time.MONTH,
    "mos": Time.MONTH,
    "mo": Time.MONTH,
    "weeks": Time.WEEK,
    "wks": Time.WEEK,
    "w": Time.WEEK,
    "days": Time.DAY,
    "d": Time.DAY,
    "hours": Time.HOUR,
    "hrs": Time.HOUR,
    "hr": Time.HOUR,
    "h": Time.HOUR,
    "minutes": Time.MINUTE,
    "mins": Time.MINUTE,
    "min": Time.MINUTE,
    "m": Time.MINUTE,  # or months, by where it sits; say_duration below decides
    "seconds": Time.SECOND,
    "secs": Time.SECOND,
    "sec": Time.SECOND,
    "s": Time.SECOND,
}
# a #, a fraction for seconds, any whitespace, and the run of english letters after them, where say_duration looks for a unit's word;
# the whitespace is unicode's white space written out, since \s and str.isspace also count four separators unicode doesn't
_duration_tags = re.compile(
    r"#(\.#{1,3})?([\t\n\v\f\r \x85\xa0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000]*)([A-Za-z]*)"
)


def say_duration(span: float, template: str) -> str:
    """Say a span of milliseconds in the units a template names, like "# hours # minutes" for "2 hours 5 minutes"; one way, with no reader."""
    span = _whole_span(span)
    _check_template(template)

    # cut the template once, into its tags and the text around them
    texts = []  # the text before each tag, and then the text after the last one
    tags = []  # each tag's unit, the whitespace and word it was written with, and how many digits of a second it wants
    at = 0  # where the text not yet cut begins
    for match in _duration_tags.finditer(template):
        fraction, whitespace, letters = match.groups()
        found = _duration_word(letters)
        if found is None:
            continue  # no unit's spelling starts the letters, so the # and any fraction it read are only text
        word, unit = found
        if fraction and unit != Time.SECOND:
            continue  # a fraction only says seconds; before any other unit, the whole match stays text, so its last # can't make a tag of its own
        texts.append(template[at : match.start()])
        tags.append(
            {
                "unit": unit,
                "whitespace": whitespace,
                "word": word,
                "digits": len(fraction) - 1 if fraction else 0,
            }
        )
        # just past the unit's word, so any letters after it stay text, like the "iring" of "#hiring"
        at = match.start(3) + len(word)
    texts.append(template[at:])
    if not tags:
        return template  # no tags, so nothing to fill in

    # a bare m is minutes, unless it sits where only months fit in a template that runs from largest to smallest: right after years, or right before weeks, days, or hours
    for i, tag in enumerate(tags):
        if tag["word"] not in ("m", "M"):
            continue
        # the unit of the tag before this one, if there is one
        before = tags[i - 1]["unit"] if i > 0 else None
        after = tags[i + 1]["unit"] if i + 1 < len(tags) else None
        if before == Time.YEAR or after in (Time.WEEK, Time.DAY, Time.HOUR):
            tag["unit"] = Time.MONTH

    # split the span among the units named, largest first, each taking all of itself that fits in what's left
    units = sorted({tag["unit"] for tag in tags}, reverse=True)
    counts = {}
    # milliseconds not yet said; a negative span says its size, and gets its minus sign at the end
    left = abs(span)
    for unit in units:
        counts[unit], left = divmod(left, unit)
    # left now holds the remainder that rounds away, or the fraction of a second

    # write each tag, then keep everything from the first tag that isn't zero through the end
    said = []
    for tag in tags:
        number = str(counts[tag["unit"]])
        if tag["digits"] > 0:
            # tenths, hundredths, or thousandths of the second, rounded down
            fraction = left // 10 ** (3 - tag["digits"])
            number += "." + str(fraction).zfill(tag["digits"])
        word = tag["word"]
        if number == "1" and len(word) > 1 and word[-1] in "sS":
            # a plural, whole or short, is singular for exactly 1, like day, hr, and DAY; the single letter s is seconds, not a plural, and never changes
            word = word[:-1]
        said.append((number + tag["whitespace"] + word, float(number) == 0))
    # zeros at the start drop away, along with the text after them
    first = next((i for i, (_, zero) in enumerate(said) if not zero), None)
    sign = ""
    if first is None:
        # every unit is zero, so say the last and smallest, like "0 minutes"
        first = len(said) - 1
    elif span < 0:
        sign = "-"  # a minus sign only in front of something that isn't zero
    text = texts[0]  # text before the first tag always stays, like "about"
    for i in range(first, len(said)):
        # from there to the end, every tag and the text after it, so the smallest unit always shows, and so does text after the last tag, like "ago"
        text += said[i][0] + texts[i + 1]
    return sign + text


def _duration_word(letters: str) -> tuple[str, int] | None:
    """The word as written and its unit, for the unit a run of letters starts with, spelled exactly in lowercase, title case, or all capitals, like days, Days, or DAYS, trying each longer word before its prefixes; or None."""
    for word, unit in _duration_units.items():
        for spelling in (word, word[0].upper() + word[1:], word.upper()):
            if letters.startswith(spelling):
                return spelling, unit
    return None


def _whole_span(span: object) -> int:
    """The whole milliseconds of a span a say function was handed, its fraction dropped toward zero so the sign stays with the size, or raise because it isn't a number within the integer ceiling; so a say function takes 1.5, which has_int refuses, and has_int(math.trunc(span)) is the question to ask first."""
    # a bool is an int in python, and never a span
    if isinstance(span, bool) or not isinstance(span, int | float):
        raise TypeError(f"not a span of milliseconds: {span!r}")
    whole = span
    # only a float has a fraction to drop, or can be infinite or not a number
    if isinstance(span, float):
        whole = math.trunc(span) if math.isfinite(span) else None
    if not has_int(whole):
        raise ValueError(f"not a span of milliseconds: {span!r}")
    return whole


def _check_template(template: object) -> str:
    """A guard: pass a template through, or raise because it isn't text; say_duration and say_tick refuse nothing else about one."""
    if not isinstance(template, str):
        raise TypeError(f"expected template text: {template!r}")
    return template


# Three shortcuts sit on top of say_duration, each with its template chosen for one use, in place of a template from the caller. They come from ergo's ancestor from 2013, which said time these three ways, and here each is a few lines of ergo.
#
# say_time_remaining says how long until something finishes, or how long it has run, in the units that matter at its size: two units while the larger is a single digit, like "9m 59s" and "1h 0m", and one unit after, like "10m" and "10h". It stays in hours up to 71 before it turns to days, since "71h" is plainer than "2d 23h" for something that finishes this week. say_time_remaining_coarse says the same, with the seconds rounded down to a multiple of 5, so a countdown on a page steps rather than flickers; the last five seconds still count down one at a time.
#
# say_time_race says how long something took to the millisecond, the way a race is timed: 5'15"223 is five minutes, fifteen seconds, and 223 milliseconds, with the minutes taking everything above them, so an hour and a quarter is 75'00"000. Its seconds and milliseconds are padded, which say_duration never does, so it writes the three numbers itself.


def say_time_remaining(span: float) -> str:
    """Say how long remains in one or two units, like "45s", "9m 59s", "10m", "1h 0m", "10h", or "3d", for a progress bar or an estimate."""
    return _say_time_remaining(span, False)


def say_time_remaining_coarse(span: float) -> str:
    """Say how long remains like say_time_remaining, with the seconds rounded down to a multiple of 5, so a countdown isn't distracting."""
    return _say_time_remaining(span, True)


def _say_time_remaining(span: float, coarse: bool) -> str:
    """Say a span with the template for the units that matter at its size."""
    span = _whole_span(span)
    # the size picks the template; a negative span gets its minus sign from say_duration
    size = abs(span)
    if coarse and size > 5 * Time.SECOND:
        # down to a multiple of 5 seconds, while the last five still count one at a time
        size -= size % (5 * Time.SECOND)
    signed = -size if span < 0 else size  # the size, with the sign of the span

    if size < Time.MINUTE:
        return say_duration(signed, "#s")  # "0s" to "59s"
    if size < 10 * Time.MINUTE:
        return say_duration(signed, "#m #s")  # "1m 0s" to "9m 59s"
    if size < Time.HOUR:
        return say_duration(signed, "#m")  # "10m" to "59m"
    if size < 10 * Time.HOUR:
        return say_duration(signed, "#h #m")  # "1h 0m" to "9h 59m"
    if size < 3 * Time.DAY:
        return say_duration(signed, "#h")  # "10h" to "71h"
    return say_duration(signed, "#d")  # "3d" and up


def say_time_race(span: float) -> str:
    """Say how long something took to the millisecond, the way a race is timed, like 5'15"223 for five minutes, fifteen seconds, and 223 milliseconds."""
    span = _whole_span(span)
    if span < 0:
        # a negative span says its size, with a minus sign in front
        return "-" + say_time_race(-span)
    minutes, rest = divmod(span, Time.MINUTE)  # the minutes take everything above them
    seconds, ms = divmod(rest, Time.SECOND)
    return f"{minutes}'{seconds:02}\"{ms:03}"  # seconds and milliseconds padded, so every time reads the same way


# tick

# A tick is an integer count of milliseconds since the start of 1970, and ergo keeps every tick between zero and the last millisecond of 9999. Two text forms carry a tick exactly. The iso form is like "2012-08-06T05:17:57.300Z": always 24 characters, always milliseconds, always Z. The short form is like "2012aug6.0517.57300": the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs, so the start of 1990 is just "1990" and the longest text is 20 characters.
#
# Each form reads back to exactly the tick that wrote it, and reads nothing else. A reader accepts the text its writer produces and refuses everything else, from a missing millisecond digit to "1990feb1" where the writer would say "1990feb". The last check in each reader is the round trip itself: parse the text, write the tick, and require the same text. That's what keeps every text one to one with its tick.
#
# Every function that takes a tick raises on a number that isn't one, because a caller handing it one has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so has_tick asks about it first, and asking never raises; once has_tick says yes, nothing here raises on that tick. check_tick is the guard that enforces the same answer, and it's the gate every function here goes through.

_last_tick = 253402300799999  # the last millisecond of 9999, which keeps a year at four digits in every edition
_epoch = datetime(1970, 1, 1, tzinfo=UTC)  # tick zero
# the months of the year and the days of the week, whole and in title case the way a sentence writes them, so a caller clips and then needs only upper() or lower() for the other two cases
# in english, each short name is the first three letters, like "Aug" and "Thu", which the short form and say_tick both clip out; tuples, so no caller can change them under either
MONTH_NAMES = (
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
)
DAY_NAMES = (
    "Sunday",  # first in every edition, the way javascript's getUTCDay and go's Weekday count from 0; python's own weekday() starts with monday, so a python caller who wants the week the other three keep has it here
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
)
# the short form's month names, clipped and lowercased once from MONTH_NAMES, like "aug"
_short_months = tuple(name[:3].lower() for name in MONTH_NAMES)
# the one shape of the iso form; [0-9] rather than \d, which would admit other scripts' digits
_iso_pattern = re.compile(
    r"([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})\.([0-9]{3})Z"
)
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
    # fullmatch rather than match with $, which would forgive a trailing newline
    m = _iso_pattern.fullmatch(s)
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
    # milliseconds into the minute, 0 through 59999
    ms = d.second * 1000 + d.microsecond // 1000
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
        text += _short_months[d.month - 1]  # the short name in lowercase, like "aug"
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
    # the round trip is the final check, catching february 30 and a padded or deeper text alike
    if t is None or tick_to_text(t) != s:
        raise ValueError(f"not the short form of a tick: {s!r}")
    return t


def _parse_text(s: str) -> int | None:
    """Read the short form leniently enough to compute a tick, or None; text_to_tick above makes it strict with the round trip."""
    # the year, month, and day; then the clock; then the milliseconds into the minute
    parts = s.split(".")
    if len(parts) > 3:
        return None
    p1 = parts[0]
    # "1990", "1990feb", "1990feb2", "1990feb28"
    if len(p1) not in (4, 7, 8, 9) or not _numerals_pattern.fullmatch(p1[:4]):
        return None
    y, m, d, h, i, ms = int(p1[:4]), 1, 1, 0, 0, 0
    if len(p1) >= 7:
        if p1[4:7] not in _short_months:
            return None
        m = _short_months.index(p1[4:7]) + 1
    if len(p1) >= 8:
        if not _numerals_pattern.fullmatch(p1[7:]):
            return None
        d = int(p1[7:])
    if len(parts) >= 2:
        p2 = parts[1]
        # "09" an hour, "0905" an hour and minute
        if len(p2) not in (2, 4) or not _numerals_pattern.fullmatch(p2):
            return None
        h = int(p2[:2])
        if len(p2) == 4:
            i = int(p2[2:])
    if len(parts) >= 3:
        if not _numerals_pattern.fullmatch(parts[2]):
            return None
        ms = int(parts[2])
    return _join_tick(y, m, d, h, i, ms)


# say_tick writes a tick for a person to read, in whatever shape a template of tags describes. "Mmm D, YYYY" says a date like "Aug 6, 2026" on a page, "YYYY-MM-DD hh24:mm:ss.ttt" stamps a line in a log, and the iso form is just "YYYY-MM-DDThh24:mm:ss.tttZ". It's the one-way companion to the two forms above, which each read back exactly. This one only writes, so a template can take any shape at all.
#
# Uppercase tags are the calendar, and lowercase tags are the clock. YYYY is the year. M and MM are the month's number, without and with a leading zero, and MMM and MMMM are its name, short and long. D and DD are the day of the month, and DDD and DDDD are the day of the week. A name comes out in the case of its tag, so MMMM is AUGUST, Mmmm is August, and mmmm is august.
#
# On the clock, h12 and h24 are the hour, each saying which clock it means, and hh12 and hh24 are the same with a leading zero. mm and ss are the minute and the second, always two digits, with no form that leaves the zero off, since five past one is 1:05 and never 1:5. t, tt, and ttt are tenths, hundredths, and thousandths of a second, as many digits as letters, rounded down so 59.999 never becomes 60. AA, aa, A, and a are PM, pm, P, and p, and it's up to the template to put them beside a twelve-hour hour.
#
# The two cases also keep the calendar and the clock apart on the page, so in "YYYY-MM-DD hh24:mm", a lowercase mm among the calendar's capitals, or a capital MM among the clock's lowercase, stands out as the mistake it is.
#
# The template is read once, left to right, taking the longest tag at each position, and whatever isn't a tag is written as it is. There's no escaping, so "YYYYY" writes the year and then a Y, like "2026Y". Each value goes straight to the output and is never read again, so a value can't make a tag: the A in AUG is only a letter. Words don't belong in a template, since a single M, D, t, A, or a is a tag, and so is a doubled mm or ss. Say the date with a small template, and put what it writes into the sentence, page, or log line around it.
#
# A fraction of a millisecond, like the one time.time() * 1000 carries, drops away, down to the millisecond the moment falls in; nothing reads this text back, so there's no exactness to keep. That means say_tick takes a number like 1.5 that has_tick refuses, since a tick is a whole number; to ask first about one that might have a fraction, ask has_tick about math.floor of it.
#
# Some things are left out on purpose. There's no reader, since the two forms above are for reading. There's no two-digit year. There are no time zones: say_tick writes the fields of the tick it's given as they fall in UTC, and works the same for a moment anywhere on earth. Keep times in UTC, and if a person should see their own wall clock, shift the tick by their offset at the very end, just before saying it. And there's no localization: the names are English, the same everywhere, and a page that wants its reader's language can hand the tick to the browser's Intl.

# every tag, each longer one before its prefixes, since alternation takes the first that matches at a position rather than the longest
_say_tags = re.compile(
    r"YYYY|MMMM|Mmmm|mmmm|MMM|Mmm|mmm|MM|M|DDDD|Dddd|dddd|DDD|Ddd|ddd|DD|D|hh12|hh24|h12|h24|mm|ss|ttt|tt|t|AA|aa|A|a"
)


def say_tick(t: float, template: str) -> str:
    """Write tick t for a person to read, in the shape a template of tags describes, like "Mmm D, YYYY" for "Aug 6, 2026"; one way, with no reader."""
    d = _split_tick(_whole_tick(t))
    _check_template(template)
    # one pass over the template; each value goes to the output and is never read again
    return _say_tags.sub(lambda m: _say_tag(d, m.group()), template)


def _whole_tick(t: object) -> int:
    """The whole millisecond a say function was handed, its fraction dropped down to the millisecond the moment falls in, or raise because that isn't a tick; so say_tick takes 1.5, which has_tick refuses, and has_tick(math.floor(t)) is the question to ask first."""
    # a bool is an int in python, and never a tick
    if isinstance(t, bool) or not isinstance(t, int | float):
        raise TypeError(f"not a tick: {t!r}")
    whole = t
    # only a float has a fraction to drop, or can be infinite or not a number
    if isinstance(t, float):
        whole = math.floor(t) if math.isfinite(t) else None
    if not has_tick(whole):
        raise ValueError(f"not a tick: {t!r}")
    return whole


def _say_tag(d: datetime, tag: str) -> str:
    """The text for one tag, from the moment in d as it falls in utc."""
    hour = d.hour  # 0 through 23
    # the hour on a twelve-hour clock, where midnight and noon are 12 rather than 0
    half = hour % 12
    if half == 0:
        half = 12
    ms = d.microsecond // 1000  # 0 through 999

    if tag == "YYYY":
        # always four digits, since a tick stays within 1970 through 9999
        return str(d.year)
    if tag == "MM":
        return f"{d.month:02}"
    if tag == "M":
        return str(d.month)
    if tag == "DD":
        return f"{d.day:02}"
    if tag == "D":
        return str(d.day)
    if tag == "hh24":
        return f"{hour:02}"
    if tag == "h24":
        return str(hour)
    if tag == "hh12":
        return f"{half:02}"
    if tag == "h12":
        return str(half)
    if tag == "mm":
        return f"{d.minute:02}"
    if tag == "ss":
        return f"{d.second:02}"
    if tag == "ttt":
        return f"{ms:03}"
    if tag == "tt":
        return f"{ms // 10:02}"  # rounded down, like t
    if tag == "t":
        return str(ms // 100)  # rounded down, so 59.99 is 59.9 and never 60.0

    # the rest are words: the month's name, the weekday's name, or am and pm, each in the case of its tag
    word = ""
    first = tag[0].lower()  # m for the month, d for the weekday, a for am and pm
    if first == "m":
        word = MONTH_NAMES[d.month - 1]
    if first == "d":
        # python counts the week from monday, 0, and ergo from sunday
        word = DAY_NAMES[(d.weekday() + 1) % 7]
    if first == "a":
        word = "AM" if hour < 12 else "PM"
    if len(tag) < 4:
        # a shorter tag writes as many letters as it has: Aug for Mmm, P for A, and both letters of PM for AA
        word = word[: len(tag)]
    return _case_like(word, tag)


def _case_like(word: str, tag: str) -> str:
    """Write a word in the case of the tag that asked for it: AUGUST for MMMM, August for Mmmm, and august for mmmm."""
    if tag == tag.upper():
        return word.upper()  # every letter a capital, like MMMM, AA, and A
    if tag == tag.lower():
        return word.lower()  # none, like mmmm, aa, and a
    return word  # only the first, like Mmmm, which is how the names are kept


# tag

# long enough to be unique, short enough to be reasonable, and nanoid's default
TAG_LENGTH = 21

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
                # 62 and 63 fall past the end; skipping them keeps the distribution uniform
                tag += _tag_alphabet[i]
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
#
# A Data can hold no bytes at all: the empty input every hash has a standard answer for, the bytes of blank text, a clip of nothing, an empty Bin. The Data in icarus, the code ergo grew from, refused nearly every way to be empty: blank text, blank encodings, zero random bytes, and a clip of nothing, and its Bin refused a capacity of zero. Only an empty array or buffer, or a Bin's data before anything was added, made an empty one there. So code built on it came to take a Data as at least one byte, and checked integrity that way. That code doesn't move to ergo's Data unchanged. Before it switches, find each place that leaned on a Data never being empty, and have it ask for itself, with size() > 0.


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
    # has_int(n) and n >= 0, the question a caller asks first, written out here since a call in python costs more than the question
    if not (isinstance(n, int) and not isinstance(n, bool) and 0 <= n <= _int_ceiling):
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


# base32, for hash values in a database, so they can be short, but also always the same length


def _bytes_to_base32(a: Buffer) -> str:
    # without the padding, as the code ergo grew from writes it
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
    """The SHA-256 hash of text, as base32, the shortcut for the common case; text to Data, hashed, and out in the form a database keeps hash values in."""
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

# The fuji hashes are two of ergo's own, from the original, the JavaScript ergo grew from, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.
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


class FujiHashes(NamedTuple):
    """The two fuji hashes, which a fuji hasher's digest answers."""

    tip_hash: Data
    piece_hash: Data


class FujiHasher:
    """A fuji hasher, a piece at a time: make one for a file of a given size, update it with each piece of the file in order, each a Data, and digest for both its tip hash and its piece hash.

    It's for bytes that arrive as events, like the chunks of an upload; hash_stream below reads a stream into one. update never fails, and digest raises ValueError when the pieces didn't add up to the size, since that's a fact about the data that came in rather than a bug. Digest doesn't finish it, as with SHA256.
    """

    __slots__ = (
        "_address",
        "_piece",
        "_piece_size",
        "_pieces",
        "_size",
        "_stripes",
        "_tips",
    )

    def __init__(self, size: int, protocol: HashProtocol = HASH_PROTOCOL) -> None:
        # measurements for the piece hash, and for the tip hash, which we'll peek for as the pieces pass; an empty file has no fuji hash, since both measures need a size of at least one byte
        measure_pieces = hash_measure(size, protocol.pieces.size)
        measure_tips = hash_measure(size, protocol.tips.size)
        self._size = size
        self._piece_size = protocol.pieces.size
        self._stripes = measure_tips.stripes

        # for the pieces hash, the summary we'll hash is the title followed by hashes of pieces of file data, each SHA-256 hash 32 bytes (hashing hashes, not file data)
        title = Data(text=f"{protocol.pieces.title}{size}.")
        self._pieces = Bin(title.size() + 32 * measure_pieces.pieces)
        self._pieces.add(title)

        # for the tips hash, the summary we'll hash is the title followed by the start, middle, and end stripes of data
        title = Data(text=f"{protocol.tips.title}{size}.")
        self._tips = Bin(title.size() + measure_tips.stripe_size)
        self._tips.add(title)

        # the piece we're filling, which a Data will look at without a copy once it's full
        self._piece = bytearray()
        self._address = 0  # how far into the file each box that arrives is

    def update(self, data: Data) -> None:
        """Hash the next piece, a Data, taking it in as the next box of bytes 📦."""
        box = _check_data(data)
        # where in the file this box of data is; then, outside the box, move address past it to mark the start of the next one
        stripe = (self._address, self._address + box.size())
        self._address += box.size()
        if self._address > self._size:
            # more bytes than the size, which digest will refuse; past the size, there's nothing to hash them into
            return

        # first, for the tip hash, we don't have random access through the file, but we do through this box!
        for start, end in hash_stripe_overlap(stripe, self._stripes):
            # subtract box address to clip in data
            self._tips.add(box.clip_view(start - stripe[0], end - stripe[0]))

        # second, for the piece hash, shovel bytes from the box into the piece until the box is empty 🪏
        shoveled = 0  # we move this index over the bytes in the box as we process them
        while shoveled < box.size():
            shovel = min(box.size() - shoveled, self._piece_size - len(self._piece))
            self._piece += box.clip_view(shoveled, shoveled + shovel)._view
            shoveled += shovel
            if len(self._piece) == self._piece_size:
                self._pieces.add(sha256(Data(array=self._piece)))
                self._piece = bytearray()

    def digest(self) -> FujiHashes:
        """Both hashes of the pieces so far, or raise because they didn't add up to the size."""
        if self._address != self._size:
            raise ValueError(
                f"the pieces were {self._address} bytes, not the {self._size} they were said to be"
            )
        # most files will end with a fragment piece smaller than a full piece; hash it into a copy of the summary, so the hasher is unchanged and digest can be asked again
        summary = Bin(self._pieces.capacity())
        summary.add(self._pieces.data())
        if self._piece:
            summary.add(sha256(Data(array=self._piece)))
        # hash the summaries of the file in the bins, which must be exactly full, a check on our prediction of how much data we'd hash
        return FujiHashes(sha256(_full(self._tips)), sha256(_full(summary)))


def hash_stream(
    stream: BinaryIO,
    size: int,
    protocol: HashProtocol = HASH_PROTOCOL,
    on_progress: Callable[[HashStatus], None] | None = None,
) -> HashStatus:
    """Given a binary stream to read and its size, compute both the tip hash and the piece hash in one pass; it reads the stream into a FujiHasher, which bytes that arrive as events can feed directly."""
    hasher = FujiHasher(size, protocol)
    status = HashStatus(total_size=size, start_time=_now(), update_time=_now())
    fed = 0  # how many bytes we've given the hasher
    # loop while boxes of data arrive from the stream; past the size, stop reading, and digest will refuse
    while fed <= size and (box := stream.read(64 * Size.KB)):
        # a new delivery of bytes from the stream has arrived! 📦
        hasher.update(Data(array=box))
        fed += len(box)
        # each update hashes every piece it fills, so this much is hashed
        hashed = fed - fed % protocol.pieces.size
        if status.hashed_size < hashed <= size:
            status.hashed_size = hashed
            status.update_time = _now()
            if on_progress:
                on_progress(status)

    # hash the summary of the file
    status.tip_hash, status.piece_hash = hasher.digest()
    status.hashed_size = size  # the fragment piece, if any, is hashed too
    status.update_time = _now()
    status.duration = status.update_time - status.start_time
    if on_progress:
        on_progress(status)
    return status

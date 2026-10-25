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

# region text
>>> from zootella.ergo import cut, deindent, has_text, trim
>>> trim("  Alice  ")  # whitespace off both ends, by unicode's list in every edition
'Alice'
>>> has_text("   ")  # nothing there but whitespace
False
>>> cut("key=value", "=")
Cut(found=True, before='key', tag='=', after='value')
>>> print(deindent('''
...     line one
...       line two
... '''), end="")  # the first line's indent comes off every line
line one
  line two

# endregion text

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

# region outline
>>> from zootella.ergo import Data, data_to_outline, deindent, text_to_outline
>>> o = text_to_outline(deindent('''
...     orchard:
...       tree:"apple"
...       basket:
...         color:"red"  # a comment, which the reader skips
... '''))  # read from text written by hand, contents in any order
>>> o.n("tree").value().text()  # n navigates to a name
'apple'
>>> o.m("basket").m("count").set_value(Data(text="12"))  # m makes as it navigates, so a chain builds as it goes
>>> print(o.text(), end="")  # written back sorted, at every level, two spaces an indent
orchard:
  basket:
    color:"red"
    count:"12"
  tree:"apple"
>>> data_to_outline(o.data()).text() == o.text()  # the bytes, the canonical form, read back to the same outline
True

# endregion outline
"""

import base64
import binascii
import functools
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


# text

# A string is a type, and text is a string that holds something. has_string asks only about the type: is this a string at all, blank and whitespace included. In Rust and Go every string parameter already is one, so there it always answers true, and it's in all four editions to keep them alike. has_text asks about what's inside: a string with at least one character that isn't whitespace, so "a" and " a " are text, while "", " ", and a tab and a newline aren't.
#
# Whitespace means one thing everywhere in ergo: the 25 characters on Unicode's White_Space list, and trim takes them off both ends of a string. Each platform's own trim keeps a list of its own. JavaScript's differs from Unicode's at exactly two characters of the 1,112,064: it trims the byte order mark, U+FEFF, which Unicode doesn't call whitespace, and keeps the next line control, U+0085, which Unicode does. Python's strip also trims the four separator controls, U+001C through U+001F, and Rust's trim and Go's TrimSpace follow Unicode's list already. So ergo uses Unicode's list in every edition, and trim, has_text, and deindent agree everywhere about what whitespace is.
#
# The two characters are worth knowing by name. Next line is a line break from IBM's mainframes, whose EBCDIC text ends each line with it, and Unicode gave it a place so that text could come over and go back. Nobody types it, and a browser never makes it. Its usual accident is the byte 0x85, the ellipsis in Windows-1252, read as Latin-1 instead, and browsers read anything labeled Latin-1 as Windows-1252. So it turns up only in old data, from a mainframe or from server code that decodes with a true Latin-1, like Python's own latin-1 codec. The byte order mark is common. It sits invisibly at the very start of a file to say how the file is encoded, and Excel's CSV export and older Windows tools write one. The web platform drops it whenever it decodes text, in fetch, TextDecoder, and a File's text, but Python's open with an encoding of utf-8 and bytes' decode keep it, unless asked for utf-8-sig, and so does a Data, so that bytes and text round-trip exactly. A server that reads an uploaded spreadsheet's bytes and splits its first line finds the mark in front of the first column's name, and trim leaves it there, as Python's own strip does. So code that reads files from people should take off a leading byte order mark on purpose.

# unicode's White_Space list, all 25, written out by code point, since python's own str.isspace and str.strip also count the four separator controls, U+001C through U+001F, which unicode doesn't
_whitespace = "".join(
    chr(c)
    for c in (
        # tab, line feed, vertical tab, form feed, and carriage return
        *range(0x09, 0x0E),
        0x20,  # space
        0x85,  # next line
        0xA0,  # no-break space
        0x1680,  # ogham space mark
        *range(0x2000, 0x200B),  # en quad through hair space, the typographer's spaces
        0x2028,  # line separator
        0x2029,  # paragraph separator
        0x202F,  # narrow no-break space
        0x205F,  # medium mathematical space
        0x3000,  # ideographic space
    )
)


def has_string(s: object) -> bool:
    """True if s is a string, blank and whitespace included; never raises."""
    return isinstance(s, str)


def check_string(s: str) -> str:
    """A guard: pass a string through, or raise because the caller handed us something else."""
    if not has_string(s):
        raise ValueError(f"not a string: {s!r}")
    return s


# /*************************************************************************/
# /*                                                                       */
# /*                                  (`-.                                 */
# /*                                   \  `                                */
# /*      /)         ,   '--.           \    `                             */
# /*     //     , '          \/          \   `   `                         */
# /*    //    ,'              ./         /\    \>- `   ,----------.        */
# /*   ( \  ,'    .-.-._        /      ,' /\    \   . `            `.      */
# /*    \ \'     /.--. .)       ./   ,'  /  \     .      `           `.    */
# /*     \     -{/    \ .)        / /   / ,' \       `     `-----.     \   */
# /*     <\      )     ).:)       ./   /,' ,' \        `.  /\)    `.    \  */
# /*      >^,  //     /..:)       /   //--'    \         `(         )    ) */
# /*       | ,'/     /. .:)      /   (/         \          \       /    /  */
# /*       ( |(_    (...::)     (                \       .-.\     /   ,'   */
# /*       (O| /     \:.::)                      /\    ,'   \)   /  ,'     */
# /*        \|/      /`.:::)                   ,/  \  /         (  /       */
# /*                /  /`,.:)                ,'/    )/           \ \       */
# /*              ,' ,'.'  `:>-._._________,<;'    (/            (,'       */
# /*            ,'  /  |     `^-^--^--^-^-^-'                              */
# /*  .--------'   /   |                                                   */
# /* (       .----'    |   *************************************************/
# /*  \ <`.  \         |   */
# /*   \ \ `. \        |   */  Make sure s is a string that has some text,
# /*    \ \  `.`.      |   */  meaning it's not blank, and not just space
# /*     \ \   `.`.    |   */
# /*      \ \    `.`.  |   */
# /*       \ \     `.`.|   */
# /*        \ \      `.`.  */
# /*         \ \     ,^-'  */
# /*          \ \    |     */
# /*           `.`.  |     */
# /*              .`.|     */
# /*               `._>    */
# /*                       */
# /*       g o o d w i n   */
# /*                       */
# /*************************/


def has_text(s: object) -> bool:
    """True if s is text: a string with at least one character that isn't whitespace, so " a " is text and " " isn't; never raises."""
    return has_string(s) and len(trim(s)) > 0


def check_text(s: str) -> str:
    """A guard: pass the text through, or raise because the caller handed us a string that's blank or just whitespace, or something that isn't a string at all."""
    if not has_text(s):
        raise ValueError(f"not text: {s!r}")
    return s  # pass the text through


def trim(s: str) -> str:
    """Take whitespace off both ends of s, by Unicode's list, which differs from Python's own strip at the four separator controls, as the essay above says."""
    # strip given a list takes off exactly those characters, from both ends
    return check_string(s).strip(_whitespace)


# cut finds a tag in text and cuts the text around it: what's before, the tag itself, and what's after, along with whether it found the tag at all. cut_last does the same from the end, cut2 finds two tags in order and the middle between them, and cut_after_last is for text whose shape you already know, answering what comes after the last tag, and raising when the tag isn't there.
#
# A tag is any string of at least one character, whitespace included, so a space, a tab, or a newline can be the tag to cut at. The blank string can't: it would be found at the start of everything, so a loop that cuts until nothing is left would never end.


class Cut(NamedTuple):
    """What cut and cut_last answer: whether they found the tag, and the text before it, the tag itself, and the text after it, which assemble back into the text."""

    found: bool
    before: str
    tag: str
    after: str


class Cut2(NamedTuple):
    """What cut2 answers: whether it found both tags, and the text before the first, the first tag, the middle between them, the second tag, and the text after it."""

    found: bool
    before: str
    tag1: str
    middle: str
    tag2: str
    after: str


def cut(s: str, tag: str) -> Cut:
    """Find the first tag in s, and cut s around it."""
    check_string(s)
    _check_cut_tag(tag)
    # python's own cut, whose middle is the tag, or blank when the tag isn't there
    before, separator, after = s.partition(tag)
    return Cut(separator != "", before, separator, after)


def cut_last(s: str, tag: str) -> Cut:
    """Find the last tag in s, and cut s around it."""
    check_string(s)
    _check_cut_tag(tag)
    before, separator, after = s.rpartition(tag)
    if separator == "":
        # not found, so all of it is before, where rpartition would put it after
        return Cut(False, s, "", "")
    return Cut(True, before, separator, after)


def cut2(s: str, tag1: str, tag2: str) -> Cut2:
    """Find tag1 in s, and then tag2 after it, and cut s around both."""
    c1 = cut(s, tag1)
    c2 = cut(c1.after, tag2)
    if c1.found and c2.found:
        return Cut2(True, c1.before, c1.tag, c2.before, c2.tag, c2.after)
    return Cut2(False, s, "", "", "", "")  # both must be found


def cut_after_last(s: str, tag: str) -> str:
    """What comes after the last tag in s, or raise because s doesn't have the tag at all."""
    c = cut_last(s, tag)
    if not c.found:
        raise ValueError(f"no {tag!r} in {s!r}")  # make sure s is what we expect
    return c.after


def _check_cut_tag(tag: str) -> str:
    """A guard: pass a tag to cut at through, or raise because it's blank or not a string; whitespace is fine."""
    if not (has_string(tag) and len(tag) > 0):
        raise ValueError(f"not a tag to cut at: {tag!r}")
    return tag


def deindent(s: str) -> str:
    """Be able to indent multiline string literals with code so they don't mess up overall readability!

    The indent of the first line comes off every line, deeper lines keep the rest of theirs, blank lines at the start and end go, and the text ends with one newline. In JavaScript it's a tag for a template literal; here it takes the text.
    """
    check_string(s)
    # temporarily normalize newlines to just \n, and split into lines there, rather than with splitlines, which would also break at the next line control, the form feed, and the line separators, which stay inside a line here
    lines = re.sub(r"\r\n|\r|\n", "\n", s).split("\n")
    # replace tabs at the start with two spaces, and take whitespace off the end
    lines = [_tabs_to_spaces(line).rstrip(_whitespace) for line in lines]
    # remove blank lines from the start and end; blank lines in the middle stay
    lines = "\n".join(lines).strip("\n").split("\n")

    first = lines[0]
    # how many spaces indent the first line
    indent = len(first) - len(first.lstrip(" "))
    # remove up to that many spaces from each line
    lines = [line[min(indent, len(line) - len(line.lstrip(" "))) :] for line in lines]
    return "\n".join(lines) + "\n"  # reassemble lines with trailing newlines


def _tabs_to_spaces(line: str) -> str:
    """Replace each tab among a line's leading spaces and tabs with two spaces."""
    return re.sub(r"^[ \t]+", lambda match: match.group().replace("\t", "  "), line)


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
# the whitespace is _whitespace, unicode's list, since \s and str.isspace also count four separators unicode doesn't
_duration_tags = re.compile(
    r"#(\.#{1,3})?([" + re.escape(_whitespace) + r"]*)([A-Za-z]*)"
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
# A Data can hold no bytes at all: the empty input every hash has a standard answer for, the bytes of blank text, a clip of nothing, an empty Bin.


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


# outline

# Outline: A tree structure where each outline has a name (string), a value (Data, empty for none), and contents (other outlines).
#
# An Outline is like JSON, but stripped down to just one type of value: bytes. No strings vs numbers vs booleans vs nulls — just a name, binary data, and contents. This simplicity makes it useful as a canonical container for cryptographic data, configuration, and structured binary payloads where type richness would be overhead.
#
# The closest inspiration is Bencoding (Bram Cohen, 2001), the serialization format inside BitTorrent .torrent files. Bencoding is minimal and deterministic — dictionaries must have sorted keys, so the same data always produces the same bytes, which is critical when the SHA-1 hash of a bencoded dictionary is the torrent's identity. Outline borrows this philosophy: deterministic sort for canonical comparison, so two Outlines with the same information produce identical binary output and identical hashes. Where Outline improves on Bencoding: a text form that is visible and useful in source code and documentation (Bencoding is barely legible for anything nontrivial), clear separation of name and value (Bencoding overloads byte strings for both keys and data), and a clean tree structure rather than Bencoding's four types (strings, integers, lists, dictionaries).
#
# MessagePack (Furuhashi, 2008), CBOR (RFC 7049, 2013), and Protocol Buffers (Google, 2008) solve a different problem. They are general-purpose serialization formats with rich type systems — integers, floats, strings, arrays, maps, booleans, nulls, extension types. Powerful, but more complex than needed when all your values are just bytes. Protobuf additionally requires schema files compiled ahead of time. Outline is deliberately simpler.
#
# Outline has two serialization formats:
# - Text: indented lines like «name:"value"0a» using quoted encoding with data_to_quoted(). Visible in source code as triple-quoted strings, explains itself in markdown documentation, diffs alongside code in version control. The parser is lenient about indentation width (2 spaces, 4 spaces, tabs), accepts trailing spaces and tabs, blank lines, and # comments at the end of a line or on a line of their own, and takes contents in any order, so pasted or hand-edited text tolerates reasonable sloppiness. The writer is canonical, sorted at every level.
# - Binary: span-prefixed length encoding (name size, name, value size, value, contents size, contents) with no delimiters or escaping. Compact and unambiguous — data stays the same size on the wire, unlike JSON which inflates binary with escape sequences. Strict, too: the reader takes only what the writer writes, sorted at every level.
#
# One value type (bytes), one tree structure, deterministic sort, a text form and a binary form. It runs anywhere Python does, with no dependencies, no schema files, and no build step.
#
# Rules for designing an outline that you're going to use in your application:
# - Names are short, lowercase, a-z and 0-9 only. Blank names are fine, duplicate names are fine.
# - Insertion order cannot matter. Sorting is deterministic and automatic, so the same information always serializes to the same bytes. If your data has ordering that matters, encode the order explicitly--for example, wrap each item in an outline named "1", "2", "3".
# - Names are labels, not data. Don't generate names from data or encode meaning in them; that's what values are for.
# - Values are bytes, as raw and granular as possible. No compression, no structured formats, no length-prefixed sub-records; if you have structure, use nested contents instead of packing it into a value.
# - No version numbers or vendor codes. An outline grows by adding new names; readers ignore names they don't recognize.
# - Outlines are short packets, not bulk storage. A typical outline is well under 8 KB as binary.
# - Outlines must be strict trees. No cycles, no shared subtrees--the same outline instance appearing in two places is refused when it's written or sorted. This matches the text form, which has no way to express sharing.

_outline_name_pattern = re.compile(r"[a-z0-9]*")


def has_outline_name(s: object) -> bool:
    """True if s is a name an outline can have: lowercase letters and numerals, or blank; never raises, so a name from outside can be asked about first."""
    # fullmatch rather than match with $, which would accept a trailing newline
    return has_string(s) and _outline_name_pattern.fullmatch(s) is not None


def check_outline_name(s: str) -> str:
    """A guard: pass an outline name through, or raise because it has a character outside a-z and 0-9, or isn't a string at all."""
    if not has_outline_name(s):
        raise ValueError(f"not an outline name: {s!r}")
    return s


# the empty Data an outline holds when it's made or cleared without a value, the same one for all of them; a reader can give an outline an empty Data of its own, which is no value just the same
_NO_VALUE = Data(array=b"")


class Outline:
    """An outline: a name, a value, and contents, which are other outlines, made with an optional name and value and filled with add and m; the essay above it in the source says what it's for and how to design one."""

    __slots__ = ("_contents", "_name", "_value")

    def __init__(self, name: str | None = None, value: Data | None = None) -> None:
        self._name = ""  # outlines can have names, [a-z0-9] only, or no name at all
        # and hold one Data value, which holds no bytes when there's no value
        self._value = _NO_VALUE
        # and hold contents, a list of lower nested outlines
        self._contents: list[Outline] = []
        # set the passed in name and value, if any
        if name is not None:
            self.set_name(name)
        if value is not None:
            self.set_value(value)

    def length(self) -> int:
        """How many outlines this one holds in its contents."""
        return len(self._contents)

    # getters and setters for the name and value of this outline
    #
    # o.name() returns o's current name
    # o.set_name("") sets o's name to blank, totally fine and useful in Outline
    # o.set_name("key1") sets o's name to "key1" only numerals and lowercase letters are allowed
    #
    # o.value() gets the value, a Data, which holds no bytes if there's no value here (could still be contents, though!)
    # o.set_value(Data(text="alice@example.com")) sets the value
    # o.set_value(None) clears the value; outlines absolutely can be nameless and/or valueless
    #
    # the name [a-z0-9] requirement may seem restrictive, but prevents the case ambiguity you've likely encountered when examining a JavaScript object of HTTP headers (did the cloud provider lowercase? could there be "origin" *and* "Origin" in here?!!) and discourages overloading values into keys (if you want dots and slashes, build that into nested contents)

    def name(self) -> str:
        """This outline's name, blank if it has none."""
        return self._name

    def set_name(self, k: str | None) -> None:
        """Set this outline's name, where None and blank are no name; raises ValueError if k isn't a name, which has_outline_name asks first."""
        # normalizing and validating through the shared helper
        self._name = _blank_or_name(k)

    def value(self) -> Data:
        """This outline's value, a Data, which holds no bytes if there's no value."""
        return self._value

    def set_value(self, p: Data | None) -> None:
        """Set this outline's value, or clear it with None."""
        if p is None:
            self._value = _NO_VALUE  # set clear
        elif isinstance(p, Data):
            self._value = p  # set to the given Data
        else:
            raise TypeError(f"an outline's value is a Data or None: {p!r}")

    def get(self, i: int) -> "Outline":
        """The outline at index i in contents; raises IndexError if there's no outline at i, which length asks first."""
        if not (
            isinstance(i, int)
            and not isinstance(i, bool)
            and 0 <= i < len(self._contents)
        ):
            raise IndexError(f"no outline {i!r} in {len(self._contents)}")
        return self._contents[i]

    def add(self, a: "Outline | str | Data") -> None:
        """Add to this outline's contents: an outline you've already built, a name for a new outline with that name and no value, or a Data for a new unnamed outline with that value. An outline already in this tree makes a cycle or a shared subtree, which sort and the writers refuse."""
        if isinstance(a, Outline):
            self._contents.append(a)  # add an outline you've already built
        elif isinstance(a, str):
            self._contents.append(Outline(a))  # add a new outline with this name
        elif isinstance(a, Data):
            # add a new unnamed outline with this value
            self._contents.append(Outline("", a))
        else:
            raise TypeError(f"an outline adds an Outline, a name, or a Data: {a!r}")

    def has(self, k: str | None = None) -> bool:
        """True if contents includes an outline with name k; raises ValueError if k isn't a name, since lookups validate too, so a mistake is a loud error instead of a silent "not found"."""
        k = _blank_or_name(k)
        return any(c.name() == k for c in self._contents)

    # o.n() for "navigate": find the first outline in contents with name k, raise if not found
    # o.m() for "make": like n, but creates the outline if it doesn't exist yet
    # o.list() returns all unnamed contents
    # o.list("type1") or filter out a named sublist, all contents named "type1"

    def n(self, k: str | None = None) -> "Outline":
        """Navigate: the first outline in contents with name k; raises KeyError if there isn't one, which has asks first, and ValueError if k isn't a name."""
        k = _blank_or_name(k)
        for c in self._contents:
            if c.name() == k:
                return c
        raise KeyError(f"no outline named {k!r}")

    def m(self, k: str | None = None) -> "Outline":
        """Make: like n, but creates the outline if it doesn't exist yet, so a chain like o.m("basket").m("color").set_value(d) builds as it goes; raises ValueError if k isn't a name."""
        k = _blank_or_name(k)
        if not self.has(k):
            self.add(Outline(k))
        return self.n(k)

    def list(self, k: str | None = None) -> "list[Outline]":
        """All the contents named k, or all the unnamed ones; raises ValueError if k isn't a name."""
        k = _blank_or_name(k)
        return [c for c in self._contents if c.name() == k]

    def remove(self, k: str | None = None) -> None:
        """Remove all outlines in contents with name k; raises ValueError if k isn't a name."""
        k = _blank_or_name(k)
        self._contents = [c for c in self._contents if c.name() != k]

    def clear(self) -> None:
        """Discard this outline's value and contents (but keep the name!)."""
        self._value = _NO_VALUE
        self._contents = []

    def sort(self) -> None:
        """Sort the contents at every level into the order the writers use, for when the order an outline holds matters, like reading it with get; the writers sort a copy of their own, so writing never needs this first. Raises ValueError if the outline holds itself, or holds the same outline in two places."""
        # refuse a cycle or a shared subtree first, as the writers do, since sorting walks the whole tree
        _outline_flat(self)

        def leave(x: Outline, depth: int) -> None:
            """Sort from the deepest levels up, since the walk leaves an outline only after all its contents, then sort this level."""
            x._contents.sort(key=_outline_order)

        _walk(self, None, leave)

    def text(self) -> str:
        """The outline as indented lines of text, its text serialization, visible in source and documentation, sorted at every level from a copy, so this outline stays as it is; raises ValueError like sort."""
        return _outline_to_text(self)

    def data(self) -> Data:
        """The outline as bytes, its binary serialization, compact on the wire, sorted the same way; raises ValueError like sort, and if a name, a value, or contents would pass the span's cap of about 256 MB."""
        return _outline_to_data(self)


def _blank_or_name(k: str | None) -> str:
    """Normalize and validate a name for the setter and all lookups: None becomes blank, and anything else must be a name."""
    if k is None:
        return ""
    return check_outline_name(k)


# text and data


def _outline_to_text(o: Outline) -> str:
    """Express an outline as indented lines of text for code, documentation, and diffs."""
    # refuse an outline that holds itself, or holds the same outline in two places, before anything walks it
    _outline_flat(o)
    # write a sorted copy, so the text is canonical and the outline the caller has stays as it is
    o = _sorted_copy(o)
    lines = []
    # the indent at each depth, two spaces per level, each made once from the one above it
    indents = [""]

    def enter(x: Outline, depth: int) -> None:
        """One line for each outline as the walk reaches it, so its contents follow it at a deeper indent."""
        if depth == len(indents):
            indents.append(indents[depth - 1] + "  ")
        lines.append(indents[depth] + x.name() + ":" + data_to_quoted(x.value()) + "\n")

    _walk(o, enter, None)
    return "".join(lines)


def text_to_outline(s: str) -> Outline:
    """Parse a text outline into an Outline, or raise ValueError because the text isn't one. It reads leniently, for text a person or an agent wrote by hand: any indent, as long as an outline's contents share one, blank lines, # comments at the end of a line or on a line of their own, and contents in any order, which it keeps."""
    check_string(s)
    # accept both \r\n and \n line endings, splitting at the line feed alone rather than with splitlines, which would also break at the other line endings unicode knows
    lines = [line.removesuffix("\r") for line in s.split("\n")]
    # skip blank lines, and lines that are only a comment, at any indent
    lines = [
        line for line in lines if has_text(line) and not re.match(r"[ \t]*#", line)
    ]
    if not lines:
        raise ValueError(f"no outline in {s!r}")

    # first pass: parse each line into (o, indent), where indent is the count of leading spaces or tabs
    # each space or tab counts as one, so 2-space, 4-space, or tab indentation all work--but must be consistent within a parent's contents
    parsed = []
    for line in lines:
        indent = re.match(r"[ \t]*", line).end()
        c = cut(line[indent:], ":")
        if not c.found:
            # every line must have a colon separating name from quoted value
            raise ValueError(f"no colon in {line!r}")

        # find where the value ends: walk forward through c.after tracking quote state, stop at the first whitespace or # outside quotes
        value_end = len(c.after)
        in_quote = False
        for j, ch in enumerate(c.after):
            if ch == '"':
                in_quote = not in_quote
                continue
            if not in_quote and ch in " \t#":
                value_end = j
                break
        value_part = c.after[:value_end]
        tail = c.after[value_end:]
        # trailing content after the value must be whitespace, or a # comment to end of line, whatever characters it holds, which DOTALL lets the dot match
        if not re.fullmatch(r"[ \t]*(#.*)?", tail, re.DOTALL):
            raise ValueError(f"more after the value in {line!r}")

        o = Outline()
        o.set_name(c.before)
        # a blank quoted form is no bytes, which is no value
        o.set_value(quoted_to_data(value_part))
        parsed.append((o, indent))

    # second pass: build the tree with a stack that tracks the current ancestry and enforces indent consistency
    # each stack entry is {o, indent, child_indent}: the outline, its own indent, and the indent its children will use (set when the first child appears, then required to match for further children)
    root, root_indent = parsed[0]
    if root_indent != 0:
        # the root must be at zero indent--no leading whitespace
        raise ValueError(f"the first line of an outline is indented: {s!r}")
    stack = [{"o": root, "indent": 0, "child_indent": None}]
    for o, indent in parsed[1:]:
        # pop entries that can't be this line's parent (at our level or deeper)
        while stack and stack[-1]["indent"] >= indent:
            stack.pop()
        if not stack:
            # outdented past the root, which means there's a second root or worse
            raise ValueError(f"more than one root in {s!r}")
        parent = stack[-1]
        if parent["child_indent"] is None:
            # first child establishes the indent for this parent's contents
            parent["child_indent"] = indent
        elif parent["child_indent"] != indent:
            # siblings must share the same indent; a mismatched indent is ambiguous
            raise ValueError(f"uneven indent in {s!r}")
        parent["o"].add(o)
        stack.append({"o": o, "indent": indent, "child_indent": None})
    return root


def _outline_to_data(o: Outline) -> Data:
    """Serialize an outline to binary data for disk or wire."""
    # refuse an outline that holds itself, or holds the same outline in two places, before anything walks it
    _outline_flat(o)
    # sort a copy depth-first before we walk the tree, so the output is deterministic, and the outline the caller has stays as it is
    o = _sorted_copy(o)

    # first pass: measure the exact size we'll need, so we can allocate one buffer and write into it
    # each outline's contents size, measured on the way out of it, so the second pass looks it up in O(1) instead of recomputing
    contents_sizes = {}
    # at each depth, the bytes measured so far of the contents of the outline at that depth
    measured = []
    total = 0

    def enter(x: Outline, depth: int) -> None:
        """Start counting the bytes of the contents of the outline the walk has reached."""
        if depth == len(measured):
            measured.append(0)
        measured[depth] = 0

    def leave(x: Outline, depth: int) -> None:
        """Measure x on the way out of it, once all its contents are measured."""
        nonlocal total
        # names are [a-z0-9], so string length equals utf-8 byte count
        name_bytes = len(x.name())
        value_bytes = x.value().size()
        # every outline in x's contents has been measured, since the walk leaves x after them
        contents_bytes = measured[depth]
        contents_sizes[id(x)] = contents_bytes
        # three (span, bytes) pairs: name, value, contents
        size = (
            len(_span_encode(name_bytes))
            + name_bytes
            + len(_span_encode(value_bytes))
            + value_bytes
            + len(_span_encode(contents_bytes))
            + contents_bytes
        )
        if depth > 0:
            # count x in the contents of the outline above it
            measured[depth - 1] += size
        else:
            total = size

    _walk(o, enter, leave)
    buffer = bytearray(total)
    w = 0  # walker, moves forward through buffer as we write bytes into it

    # second pass: write the bytes in the same order we just measured
    def write(b: bytes) -> None:
        """Write bytes into the buffer, advancing the walker."""
        nonlocal w
        buffer[w : w + len(b)] = b
        w += len(b)

    def compose(x: Outline, depth: int) -> None:
        """Write x's name, value, and contents span as the walk reaches it, so its contents follow."""
        name = x.name().encode()
        value = x.value().array()
        write(_span_encode(len(name)))  # name span and bytes
        write(name)
        write(_span_encode(len(value)))  # value span and bytes
        write(value)
        # contents span, then each contained outline, which the walk reaches next
        write(_span_encode(contents_sizes[id(x)]))

    _walk(o, compose, None)
    return Data(array=bytes(buffer))


def data_to_outline(d: Data) -> Outline:
    """Parse bytes into an Outline, or raise ValueError because they aren't exactly what Outline.data writes: contents sorted at every level, every span in its shortest form, and nothing after. It's strict, for bytes from the wire, and a program reading from the wire caps how many bytes it accepts first."""
    a = _check_data(d).array()
    w = 0  # walker, moves forward through a as we read bytes out of it

    def span() -> int:
        """Read a size prefix, advancing the walker."""
        nonlocal w
        value, bytes_read = _span_decode(a, w)
        w += bytes_read
        return value

    def read(n: int) -> bytes:
        """Read n bytes, advancing the walker."""
        nonlocal w
        if w + n > len(a):
            raise ValueError(f"an outline needs {n} bytes at {w} of {len(a)}")
        b = a[w : w + n]
        w += n
        return b

    def start() -> dict:
        """Start parsing one outline: name span + bytes, value span + bytes, then the contents span, which says where its contents end."""
        o = Outline()

        name_size = span()
        if name_size > 0:
            # a name's bytes are its text, which the setter checks is a-z and 0-9
            o.set_name(Data(array=read(name_size)).text())

        value_size = span()
        if value_size > 0:
            # zero size stays as the default, no value
            o.set_value(Data(array=read(value_size)))

        contents_size = span()
        # previous will be the outline parsed just before the next one in its contents
        return {"o": o, "end": w + contents_size, "previous": None}

    # parse outlines inside outlines with a stack of our own, rather than calling ourselves for each level down: the outlines still being parsed, from the root down, each with where its contents end
    open_outlines = [start()]
    root = open_outlines[0]["o"]
    while open_outlines:
        top = open_outlines[-1]
        if w < top["end"]:
            # parse contained outlines until we've consumed exactly contents_size bytes
            open_outlines.append(start())
            continue
        if w != top["end"]:
            # overran the contents region, since a reader short of the end goes on to the next outline inside, so the input is malformed
            raise ValueError("an outline's contents ran past their size")

        # top is whole, its contents all parsed, so now it can be compared and added to the outline above it
        open_outlines.pop()
        if not open_outlines:
            break  # top is the root
        parent = open_outlines[-1]
        previous = parent["previous"]
        if previous is not None and _outline_compare(previous, top["o"]) > 0:
            # the binary form is the canonical one, sorted at every level, so the same information is always the same bytes, and an outline's hash is its identity
            raise ValueError("the contents of an outline in binary are out of order")
        parent["o"].add(top["o"])
        parent["previous"] = top["o"]
    if w != len(a):
        # trailing bytes after the root outline mean malformed input
        raise ValueError(f"{len(a) - w} bytes after the outline")
    return root


# Every walk through an outline keeps a stack of its own, rather than calling itself for each level down. The writers, sort, and the check for cycles walk with _walk, which calls enter with each outline as it reaches it, and leave once all of that outline's contents are done, so what a recursive function would do before calling itself goes in enter, and what it would do after, in leave. Compare keeps a stack of the pairs it has yet to compare, and the binary reader a stack of the outlines it's still reading, each with where its contents end. The text reader has kept a stack of the outlines above the line it's reading since it was first written.
#
# The other walks were written recursively at first, which reads more simply, and moved to loops when depth came to matter. A walk that calls itself spends a frame of the language's own stack on every level, and that stack is small: a few thousand levels in JavaScript and in Rust, about a thousand in Python. An outline's binary form arrives from the wire, so a few kilobytes of nesting from anyone could run it out. Past that point, JavaScript and Python throw, Go, whose stacks grow to a gigabyte before it gives up, stops with a fatal error, and Rust aborts the whole process, which nothing can catch. A walk with a stack of its own needs only memory, so how deep an outline goes is limited by memory alone, how large its bytes can be by the span's cap, and no nesting from anywhere can crash it. In JavaScript, timed against the recursion they replaced, the loops are no slower.


def _walk(
    root: Outline,
    enter: Callable[[Outline, int], None] | None,
    leave: Callable[[Outline, int], None] | None,
) -> None:
    """Walk an outline depth first: call enter with each outline as the walk reaches it, and how many levels down it is, and leave with each outline and its depth after all its contents; either can be None."""
    path = [root]  # the outlines from the root down to where the walk is
    # and at each level, the index of the next outline to visit in its contents
    following = [0]
    if enter:
        enter(root, 0)
    while path:
        level = len(path) - 1
        o = path[level]
        if following[level] < o.length():
            # go down into the next outline in contents
            c = o.get(following[level])
            following[level] += 1
            path.append(c)
            following.append(0)
            if enter:
                enter(c, level + 1)
        else:
            path.pop()  # or back up, having visited them all
            following.pop()
            if leave:
                leave(o, level)


def _sorted_copy(o: Outline) -> Outline:
    """A copy of an outline with its contents sorted at every level, deepest first, for the writers; the values are shared rather than copied, since a Data's bytes are the same either way."""
    copies = []  # at each depth, the copy the walk is making there
    contents = []  # and the copies of its contents, gathered to sort before they go in

    def enter(x: Outline, depth: int) -> None:
        """Make a copy of the outline the walk has reached, and start gathering copies of its contents."""
        if depth == len(copies):
            copies.append(None)
            contents.append(None)
        copies[depth] = Outline(x.name(), x.value())
        contents[depth] = []

    def leave(x: Outline, depth: int) -> None:
        """Sort from the deepest levels up, since the walk leaves an outline only after all its contents, then sort this level."""
        contents[depth].sort(key=_outline_order)
        for c in contents[depth]:
            copies[depth].add(c)
        if depth > 0:
            # and gather this copy, now whole, for the copy above it
            contents[depth - 1].append(copies[depth])

    _walk(o, enter, leave)
    return copies[0]


def _outline_flat(o: Outline) -> None:
    """Make sure no outline appears twice in o's tree, in a cycle or as a shared subtree."""
    seen = set()

    def enter(x: Outline, depth: int) -> None:
        """Refuse an outline the walk has already reached once."""
        if id(x) in seen:
            # cycle or DAG: x already appears somewhere else in the tree, which the walk finds the first time it comes back to x, before it can go around again
            raise ValueError(
                "an outline appears twice in its tree, in a cycle or as a shared subtree"
            )
        seen.add(id(x))

    _walk(o, enter, None)


def _outline_compare(o1: Outline, o2: Outline) -> int:
    """Compare two outlines for sort order: name first, then value, then nested contents pairwise down the whole tree; pairwise gives the canonical order only when the levels below are already in it, so every caller sorts or checks them first."""
    c = _outline_compare_here(o1, o2)
    if c != 0:
        # names or values differ, which settles nearly every comparison a sort makes, with nothing made to do it
        return c

    # value tied, so we examine contents pairwise, with a stack of our own of the pairs still to compare, rather than calling ourselves for each level down
    # the pair on top is next; a pair marked counts compares how many contents its two outlines hold, once all the pairs of their contents have tied
    pending = []

    def push(a: Outline, b: Outline) -> None:
        """Compare contents pairwise at each index, the first pair on top, so it and everything inside it is compared before the second, and their counts after."""
        pending.append((a, b, True))
        for i in range(min(a.length(), b.length()) - 1, -1, -1):
            pending.append((a.get(i), b.get(i), False))

    push(o1, o2)
    while pending:
        a, b, counts = pending.pop()
        if counts:
            if a.length() < b.length():
                return -1  # all shared contents match, fewer contents sorts first
            if a.length() > b.length():
                return 1
        else:
            c = _outline_compare_here(a, b)
            if c != 0:
                # every caller sorts or checks deeper levels first, so contents at each level are already in order
                return c
            push(a, b)
    return 0  # a complete and deep tie: identical name, value, and contents


def _outline_compare_here(o1: Outline, o2: Outline) -> int:
    """Compare two outlines by their names and values alone, for compare as it goes down the tree."""
    # name first
    if o1.name() < o2.name():
        return -1  # o1 wins with a lighter name
    if o1.name() > o2.name():
        return 1  # o2 wins with its lighter name

    # same name, so we move on to compare value; no value is an empty Data, which sorts before any value, since a shorter value sorts first
    # python compares bytes the way an outline sorts them: byte by byte up to the shorter length, where the first differing byte decides the order, and then the shorter value first
    a1 = o1.value().array()
    a2 = o2.value().array()
    if a1 < a2:
        return -1
    if a1 > a2:
        return 1
    return 0  # value tied, so compare goes on to examine contents pairwise


# the order sort and the sorted copy put contents in, as the key python's sort takes
_outline_order = functools.cmp_to_key(_outline_compare)


# span

# Span: a variable-length integer encoding for size prefixes in the binary format. Each byte contributes 7 bits of value; the high bit is 1 to continue, 0 to stop. Least-significant byte first (little-endian), matching LEB128 (WebAssembly, DWARF debug info) and Protocol Buffers' varint. Capped at 4 bytes (28 bits), giving a maximum value of 268,435,455 — about 256 MB. Outlines are designed to be small packets, not bulk storage, so this ceiling is generous. An Outline that approaches this size is almost certainly a misuse.
#
# An outline's name, its value, and its contents each come after a span of their size, the root's included, so the cap holds each of them to about 256 MB: the writer refuses an outline with a name, a value, or contents past it, and no span can tell the reader to expect more, though the root's three together can come to about three times that. That's still more than a program should take from outside. An outline takes as few as 3 bytes in its binary form and about 120 in memory, so reading bytes can take some 40 times their size, and running out of memory can end the process before Python gets the chance to raise a MemoryError. So a program that reads outlines from the wire caps how many bytes it accepts before it hands them to data_to_outline, the way a server caps the size of a request body.


class Span(NamedTuple):
    """What span_decode answers: the size the span says, and how many bytes the span took, so a reader can move past it."""

    value: int
    bytes_read: int


def span_encode(n: int) -> Data:
    """A size as a span, a Data of 1 to 4 bytes, the way an outline's binary form writes each size."""
    return Data(array=_span_encode(n))


def span_decode(data: Data, offset: int) -> Span:
    """Read the span at an offset into a Data, answering its value and how many bytes it took, so a reader can move past it."""
    return _span_decode(_check_data(data).array(), _check_count(offset))


def _span_encode(n: int) -> bytes:
    """A size as a span's bytes, at most 4."""
    # has_int(n) and n >= 0 and under the cap, written out the way _check_count is; max 268,435,455 (~256 MB)
    if not (isinstance(n, int) and not isinstance(n, bool) and 0 <= n < 0x10000000):
        raise ValueError(f"not a size a span can hold: {n!r}")
    b = bytearray()
    while True:
        y = n & 0x7F  # take the lowest 7 bits
        n >>= 7  # shift them off, ready to take the next 7
        if n > 0:
            # set the high bit to signal "more bytes follow" to the decoder
            y |= 0x80
        b.append(y)
        if n == 0:
            return bytes(b)


def _span_decode(a: bytes, offset: int) -> Span:
    """Answer the span's value and how many bytes it took, so the caller can advance its cursor."""
    n = 0
    shift = 0
    i = offset
    # at most 4 bytes; more than that means malformed input
    for count in range(4):
        if i >= len(a):
            raise ValueError(f"a span runs past the end at {i}")
        y = a[i]
        i += 1
        n |= (y & 0x7F) << shift  # accumulate the 7 value bits at their proper place
        if y & 0x80 == 0:
            # high bit clear means this is the last byte
            if count > 0 and y == 0:
                # reject non-canonical encoding: a multi-byte span whose top byte contributes no bits could have fit in fewer bytes
                raise ValueError(
                    f"a span written longer than it needs to be at {offset}"
                )
            return Span(n, i - offset)
        shift += 7
    # ran past the 4 byte limit without finding a terminating byte
    raise ValueError(f"a span longer than 4 bytes at {offset}")


# quoted encoding

# "Quoted" Encoding: A text format for binary data that mixes quoted ASCII text with base16
#
# «"Hello"0a» is easier to read than base16 '48656c6c6f0a'
# «"A"00» shows the bits of a null terminator in a language-agnostic way, unlike 'A\0'
# «» blank text is valid quoted encoding to express nothing, size zero bytes
#
# unlike the base encoding of Data, quoted is *not* round-trip!
# a heuristic below balances brevity and readability when choosing which ASCII bytes to expose in quotes


def _quotable(c: int) -> bool:
    """True if the given character c is understandable to read in quotes: space through tilde, but not the double quotation mark itself, which is our delimiter!"""
    return 0x20 <= c <= 0x7E and c != 0x22


def data_to_quoted(data: Data) -> str:
    """Encode the given Data as text in quoted encoding, where no bytes at all are blank text."""
    a = _check_data(data).array()

    text_count = sum(1 for c in a if _quotable(c))
    if text_count * 2 < len(a):
        # under half quotable? it's random or binary, just use base16; ties go to quoted
        return data.base16()

    s = []
    i = 0
    # alternate between:
    while i < len(a):
        if _quotable(a[i]):
            # runs of quotable text, and
            start = i
            while i < len(a) and _quotable(a[i]):
                i += 1
            s.append('"' + a[start:i].decode("ascii") + '"')
        else:
            # runs of unquotable bytes in base16
            start = i
            while i < len(a) and not _quotable(a[i]):
                i += 1
            s.append(a[start:i].hex())
    return "".join(s)


def quoted_to_data(s: str) -> Data:
    """Convert quoted text back to Data, where blank text is valid quoted encoding to describe zero bytes of nothing."""
    check_string(s)
    b = _quoted_bytes(s)
    if b is None:
        raise ValueError(f"not quoted encoding: {s!r}")
    return Data(array=b)


# the characters of lowercase base16, which is all quoted encoding takes outside quotes
_base16_digits = "0123456789abcdef"


def _quoted_bytes(s: str) -> bytes | None:
    """The bytes quoted text holds, or None when it isn't quoted encoding."""
    # walk through the string, alternating between quoted sections and base16 pairs
    b = bytearray()
    i = 0
    while i < len(s):
        if s[i] == '"':
            # opening quote starts a text section
            i += 1
            close = s.find('"', i)
            if close == -1:
                return None  # no closing quote
            for ch in s[i:close]:
                # strict: every character inside must be something data_to_quoted would have quoted
                c = ord(ch)
                if not _quotable(c):
                    # rejects control chars, high bytes, unicode, and the quote char itself
                    return None
                b.append(c)  # quotable guarantees ASCII, one byte per char
            i = close + 1
        else:
            # outside quotes, consume base16 digit pairs
            if i + 1 >= len(s):
                return None  # a lone base16 digit with no pair is invalid
            c0 = s[i]
            c1 = s[i + 1]
            if c0 not in _base16_digits or c1 not in _base16_digits:
                return None  # lowercase base16 only
            b.append(int(c0 + c1, 16))
            i += 2
    return bytes(b)

"""The fuzz tests: random ticks through both text forms and say_tick, and random tags, hashes, and outlines, as fast as this edition can, for the number of seconds ERGO_FUZZ names.

Skipped otherwise, so the suite stays deterministic and fast. Run with ERGO_FUZZ=4 uv run pytest -s -k fuzz, where -s shows the count it reached.
"""

import os
import random
import time

import pytest

from zootella.ergo import (
    BLAKE3,
    SHA1,
    SHA256,
    Data,
    Outline,
    Size,
    Time,
    blake3,
    data_to_outline,
    data_to_quoted,
    has_tag,
    iso_to_tick,
    say_tick,
    sha1,
    sha256,
    span_decode,
    span_encode,
    tag,
    text_to_outline,
    text_to_tick,
    tick_to_iso,
    tick_to_text,
)

seconds = float(os.environ.get("ERGO_FUZZ", "0"))

# centers and radii from the tick's original fuzz test: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range
centers = [
    ("2005", 30 * Time.YEAR),
    ("2000feb29", 3 * Time.DAY),
    ("2000", Time.HOUR),
    ("2000mar", Time.HOUR),
    ("2000jan1.12", 6 * Time.HOUR),
    ("2000jan1.1230", 30 * Time.MINUTE),
    ("2000jan1.1230.30000", 30 * Time.SECOND),
    ("2000", 15 * Time.DAY),
    ("2000mar", 15 * Time.DAY),
    ("2000jan1.12", 15 * Time.DAY),
    ("2000jan1.1230", 12 * Time.HOUR),
    ("2000jan1.1230.30000", 30 * Time.MINUTE),
    ("1970", Time.DAY),
    ("9999dec31", Time.DAY),
]
# the ceiling, so a tick drawn near an edge center stays a tick
last_tick = text_to_tick("9999dec31.2359.59999")
# a template that says every field of a tick, the weekday among them
every_field = "Dddd YYYY-MM-DD hh12:mm:ss.ttt AA"


@pytest.mark.skipif(seconds <= 0, reason="set ERGO_FUZZ to a number of seconds")
def test_fuzz_tick() -> None:
    budget = seconds / len(centers)  # seconds to spend on each center
    cycles = 0
    for center, radius in centers:
        c = text_to_tick(center)
        stop = time.perf_counter() + budget
        while time.perf_counter() < stop:
            # a random tick within the radius, clamped to the range of ticks
            t = min(max(c + random.randint(-radius, radius), 0), last_tick)
            assert text_to_tick(tick_to_text(t)) == t
            assert iso_to_tick(tick_to_iso(t)) == t
            # and say it with every field, as the fuzz across the editions does, which is where what it says gets compared
            say_tick(t, every_field)
            cycles += 1
    print(
        f"\npython fuzzed {cycles} ticks through both text forms and say_tick in {seconds:g} seconds, {round(cycles / seconds)} a second"
    )


@pytest.mark.skipif(seconds <= 0, reason="set ERGO_FUZZ to a number of seconds")
def test_fuzz_tag() -> None:
    """A hot loop making tags: the smoke test that this edition makes them as fast as it can."""
    stop = time.perf_counter() + seconds
    count = 0
    while time.perf_counter() < stop:
        assert has_tag(tag())
        count += 1
    print(
        f"\npython made {count} tags in {seconds:g} seconds, {round(count / seconds)} a second, each checked with has_tag"
    )


@pytest.mark.skipif(seconds <= 0, reason="set ERGO_FUZZ to a number of seconds")
def test_fuzz_hash() -> None:
    """Each hash as fast as this edition can, on one mebibyte of random bytes again and again, and once a piece at a time to check it agrees."""
    data = Data(random=Size.MB)
    hashes = [
        ("sha1", sha1, SHA1),
        ("sha256", sha256, SHA256),
        ("blake3", blake3, BLAKE3),
    ]
    share = seconds / len(hashes)  # an even share of the seconds for each hash
    said = []
    for name, f, hasher in hashes:
        stop = time.perf_counter() + share
        last, count = f(data), 1  # at least once, however few the seconds
        while time.perf_counter() < stop:
            last = f(data)
            count += 1
        h = hasher()
        # and once a piece at a time, at a split that lines up with nothing, which must agree
        h.update(data.clip_view(0, 1000))
        h.update(data.clip_view(1000, data.size()))
        assert h.digest() == last
        said.append(f"{name} {round(count / share)} MiB a second")
    print(f"\npython hashed one mebibyte again and again: {', '.join(said)}")


# the names random outlines take, a few of them, blank most of all, so siblings share names and ties come up
outline_names = ["", "", "", "a", "a", "b", "ab", "a0", "z9"]
# and the values, as base16: none most of all, and ones that tie, that begin one another, and that are the characters quoted encoding treats with care, a quote, a space, and a number sign
outline_values = ["", "", "", "00", "01", "61", "6161", "22", "20", "23"]
# and the characters of a value that reads as text, with the ones a text outline treats with care
outline_characters = list('ab:# "\t\n')


def random_value() -> Data:
    """A random value: often one from the list above, sometimes text, sometimes random bytes."""
    r = random.random()
    if r < 0.6:
        return Data(base16=random.choice(outline_values))
    if r < 0.8:
        return Data(
            text="".join(random.choices(outline_characters, k=random.randrange(12)))
        )
    return Data(random=random.randrange(40))


def random_outline(depth: int) -> Outline:
    """A random outline, its contents nested down at most depth levels."""
    o = Outline(random.choice(outline_names), random_value())
    if depth > 0:
        for _ in range(random.randrange(5)):
            o.add(random_outline(depth - 1 - random.randrange(2)))
    return o


def outline_shape(o: Outline) -> tuple:
    """An outline as plain data, in the order it holds its contents, to compare two outlines, or one before and after."""
    return (
        o.name(),
        o.value().base16(),
        tuple(outline_shape(o.get(i)) for i in range(o.length())),
    )


def shuffled_outline(o: Outline) -> Outline:
    """A copy of an outline with its contents in a random order at every level."""
    copy = Outline(o.name(), o.value())
    for i in random.sample(range(o.length()), o.length()):
        copy.add(shuffled_outline(o.get(i)))
    return copy


def sloppy_quoted(d: Data) -> str:
    """A value in quoted encoding a person might write instead: as the writer writes it, as base16 alone, or with quotable bytes quoted one at a time, or not."""
    r = random.random()
    if r < 0.4:
        return data_to_quoted(d)
    if r < 0.7:
        return d.base16()
    s = ""
    for b in d.array():
        if 0x20 <= b <= 0x7E and b != 0x22 and random.random() < 0.5:
            s += '"' + chr(b) + '"'
        else:
            s += f"{b:02x}"
    return s


def sloppy_text(o: Outline) -> str:
    """An outline as text the way a person might write it by hand: contents in any order, any indent, any quoting, trailing whitespace and comments, lines of comments, blank lines, and either line ending."""
    lines = []

    def walk(o: Outline, indent: str) -> None:
        line = indent + o.name() + ":" + sloppy_quoted(o.value())
        line += random.choice(
            [
                "",
                "",
                " ",
                "\t",
                " #note",
                "#",
                ' # "quoted" #more',
                "\t#" + chr(0x2028) + "\r",
            ]
        )
        lines.append(line + random.choice(["\n", "\r\n"]))
        if random.random() < 0.1:
            # a blank line, of nothing or of whitespace
            lines.append(random.choice(["", "  ", "\t", chr(0x3000)]) + "\n")
        if random.random() < 0.1:
            # a line that's only a comment, at any indent
            lines.append(
                random.choice(["", " ", "\t", indent])
                + random.choice(["#", "# a note", '#x:"commented out"'])
                + "\n"
            )
        # each parent picks the indent its own contents share
        unit = random.choice(["  ", "    ", "\t", " ", " \t"])
        for i in random.sample(range(o.length()), o.length()):
            walk(o.get(i), indent + unit)

    walk(o, "")
    return "".join(lines)


def mutated_data(d: Data) -> Data:
    """A copy of some bytes with one thing changed: a byte replaced, inserted, or deleted, or the end cut off."""
    a = bytearray(d.array())
    i = random.randrange(len(a) + 1)
    b = random.choice(
        [random.randrange(256), random.choice([0x00, 0x01, 0x30, 0x61, 0x62, 0x80])]
    )
    r = random.random()
    if r < 0.4 and a:
        a[min(i, len(a) - 1)] = b
    elif r < 0.6:
        a.insert(i, b)
    elif r < 0.8 and a:
        del a[min(i, len(a) - 1)]
    else:
        del a[i:]
    return Data(array=bytes(a))


def mutated_text(s: str) -> str:
    """And a copy of some text with one character replaced, inserted, or deleted, from the characters a text outline cares about."""
    i = random.randrange(len(s) + 1)
    c = random.choice([*'a:" \t#\n0A\r', chr(0xA0)])
    r = random.random()
    if r < 0.4:
        return s[:i] + c + s[i + 1 :]
    if r < 0.7:
        return s[:i] + c + s[i:]
    return s[:i] + s[i + 1 :]


@pytest.mark.skipif(seconds <= 0, reason="set ERGO_FUZZ to a number of seconds")
def test_fuzz_outline() -> None:
    """A random outline round-trips through both forms, and writes the same bytes from any order and any hand-written text."""
    stop = time.perf_counter() + seconds
    cycles = 0
    while time.perf_counter() < stop:
        o = random_outline(5)
        before = outline_shape(o)
        d = o.data()  # the canonical bytes
        t = o.text()  # and the canonical text
        assert outline_shape(o) == before, "a writer changed the outline"
        assert data_to_outline(d).data().base16() == d.base16(), t
        assert data_to_outline(d).text() == t, t
        assert text_to_outline(t).data().base16() == d.base16(), t
        shuffled = shuffled_outline(o)
        assert shuffled.data().base16() == d.base16(), t
        shuffled.sort()
        assert outline_shape(shuffled) == outline_shape(data_to_outline(d)), t
        sloppy = sloppy_text(o)
        assert text_to_outline(sloppy).data().base16() == d.base16(), repr(sloppy)
        cycles += 1
    print(
        f"\npython fuzzed {cycles} random outlines through both forms, in any order, and as hand-written text, in {seconds:g} seconds, {round(cycles / seconds)} a second"
    )


@pytest.mark.skipif(seconds <= 0, reason="set ERGO_FUZZ to a number of seconds")
def test_fuzz_outline_readers() -> None:
    """The readers take only what reads back the same: changed bytes, text, and spans refuse, or read to a form that round-trips."""
    stop = time.perf_counter() + seconds
    cycles = 0
    refused = 0
    while time.perf_counter() < stop:
        o = random_outline(3)

        # the binary reader is strict: whatever bytes it takes are exactly the bytes its writer writes
        d = mutated_data(o.data())
        try:
            from_data = data_to_outline(d)
        except ValueError:
            refused += 1  # refused, which is fine; the property is about what it takes
        else:
            assert from_data.data().base16() == d.base16(), d.base16()

        # the text reader is lenient, but whatever text it takes, it writes back as text that reads to the same outline
        s = mutated_text(o.text())
        try:
            from_text = text_to_outline(s)
        except ValueError:
            refused += 1
        else:
            t = from_text.text()
            assert text_to_outline(t).text() == t, repr(s)
            assert text_to_outline(t).data().base16() == from_text.data().base16(), (
                repr(s)
            )

        # and a span is strict, too: a size writes as one set of bytes, and reads back from them, and no other bytes read to it
        # as often small as large, up to the most four bytes hold
        n = random.randrange(2 ** random.randrange(29))
        e = span_encode(n)
        assert span_decode(e, 0) == (n, e.size()), n
        x = Data(random=random.randint(1, 5))
        try:
            from_span = span_decode(x, 0)
        except ValueError:
            refused += 1
        else:
            assert (
                span_encode(from_span.value).base16()
                == x.clip_view(0, from_span.bytes_read).base16()
            ), x.base16()
        cycles += 1
    print(
        f"\npython changed {cycles} random outlines' bytes and text, and read random spans, in {seconds:g} seconds, {round(cycles / seconds)} a second, and the readers refused {refused} of the {cycles * 3}"
    )

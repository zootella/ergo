"""The fuzz test: random ticks through both text forms and say_tick, as fast as this edition can, for the number of seconds ERGO_FUZZ names.

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
    Size,
    Time,
    blake3,
    has_tag,
    iso_to_tick,
    say_tick,
    sha1,
    sha256,
    tag,
    text_to_tick,
    tick_to_iso,
    tick_to_text,
)

seconds = float(os.environ.get("ERGO_FUZZ", "0"))

# centers and radii from cold3: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range
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

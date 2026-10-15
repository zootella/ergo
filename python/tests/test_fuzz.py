"""The fuzz test: random ticks through both text forms, as fast as this edition can, for the number of seconds ERGO_FUZZ names.

Skipped otherwise, so the suite stays deterministic and fast. Run with ERGO_FUZZ=4 uv run pytest -s -k fuzz, where -s shows the count it reached.
"""

import os
import random
import time

import pytest

from zootella.ergo import (
    Time,
    has_tag,
    iso_to_tick,
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
            cycles += 1
    print(
        f"\npython fuzzed {cycles} ticks through both text forms in {seconds:g} seconds, {round(cycles / seconds)} a second"
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

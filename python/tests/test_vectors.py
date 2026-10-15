"""The vector reader: run every case in the shared test vectors against this edition."""

import json
from pathlib import Path

import pytest

from zootella.ergo.driver import call

vectors_directory = (
    Path(__file__).parent.parent.parent / "tests"
)  # the shared vectors beside this edition
cases = [
    c
    for f in sorted(vectors_directory.glob("*.json"))
    for c in json.loads(f.read_text())
]


def describe(c: dict) -> str:
    """Name a case the way it reads: saySize4(10000) five digits of bytes, so the next unit up."""
    return f"{c['function']}({', '.join(json.dumps(a) for a in c['input'])}) {c.get('note', '')}"


@pytest.mark.parametrize("c", cases, ids=describe)
def test_vector(c: dict) -> None:
    if c.get("throws"):
        with pytest.raises(Exception):  # noqa: B017, a throws vector means this edition fails loudly in its own way; which exception is its business, not the vector's
            call(c["function"], c["input"])
    else:
        assert call(c["function"], c["input"]) == c["output"]

"""The python edition's driver: one json request per line on stdin, one json response per line on stdout.

Also the dispatch table from the protocol's function names, which are the JavaScript edition's, to this edition's. A constant is a function with no arguments, and a dotted name like Size.kb is a constant in a namespace. Bytes cross as tagged json: {"base16": "616263"} is those three bytes, and {"repeat": "61", "length": 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line. The vector reader in tests uses the table too.
"""

import json
import re
import sys

from zootella.ergo import (
    BLAKE3,
    SHA1,
    SHA256,
    TAG_LENGTH,
    Size,
    Time,
    blake3,
    check_tag,
    check_tick,
    has_tag,
    has_tick,
    iso_to_tick,
    say_size4,
    sha1,
    sha256,
    tag,
    text_to_tick,
    tick_to_iso,
    tick_to_text,
)

functions = {
    "Size.b": Size.B,
    "Size.kb": Size.KB,
    "Size.mb": Size.MB,
    "Size.gb": Size.GB,
    "Size.tb": Size.TB,
    "Size.pb": Size.PB,
    "saySize4": say_size4,
    "Time.millisecond": Time.MILLISECOND,
    "Time.second": Time.SECOND,
    "Time.minute": Time.MINUTE,
    "Time.hour": Time.HOUR,
    "Time.day": Time.DAY,
    "Time.week": Time.WEEK,
    "Time.year": Time.YEAR,
    "Time.month": Time.MONTH,
    "hasTick": has_tick,
    "checkTick": check_tick,
    "tickToText": tick_to_text,
    "textToTick": text_to_tick,
    "tickToIso": tick_to_iso,
    "isoToTick": iso_to_tick,
    "tagLength": TAG_LENGTH,
    "Tag": tag,
    "hasTag": has_tag,
    "checkTag": check_tag,
    "sha1": sha1,
    "sha256": sha256,
    "blake3": blake3,
    "Sha1": lambda pieces: _fed(SHA1(), pieces),
    "Sha256": lambda pieces: _fed(SHA256(), pieces),
    "Blake3": lambda pieces: _fed(BLAKE3(), pieces),
}


def _fed(h: SHA1 | SHA256 | BLAKE3, pieces: list[bytes]) -> bytes:
    """A hasher in the protocol takes one argument, a list of pieces, and answers the hash of all of them fed in order, so a vector pins the piece-at-a-time form without the harness keeping an object between calls."""
    for piece in pieces:
        h.update(piece)
    return h.digest()


def _from_protocol(v: object) -> object:
    """An argument as this edition takes it: tagged bytes become bytes, a list is decoded all through, and anything else passes as it is."""
    if isinstance(v, list):
        return [_from_protocol(x) for x in v]
    if isinstance(v, dict) and "base16" in v:
        return _base16_to_bytes(v["base16"])
    if isinstance(v, dict) and "repeat" in v:
        unit, n = _base16_to_bytes(v["repeat"]), v.get("length")
        # a unit of at least one byte, repeated to a count within the integer ceiling, the same rule in every edition; a bool is an int in python, and never a count
        if (
            not unit
            or not isinstance(n, int)
            or isinstance(n, bool)
            or not 0 <= n <= 2**53 - 1
        ):
            raise ValueError(f"not bytes: {v!r}")
        return (unit * (n // len(unit) + 1))[:n]
    return v


_base16_pattern = re.compile(r"(?:[0-9a-fA-F]{2})*")


def _base16_to_bytes(s: object) -> bytes:
    """Bytes from base16, strictly: an even number of the digits 0 through 9 and a through f, in either case, and nothing else, where fromhex alone would also skip spaces."""
    if not isinstance(s, str) or not _base16_pattern.fullmatch(s):
        raise ValueError(f"not base16: {s!r}")
    return bytes.fromhex(s)


def _to_protocol(v: object) -> object:
    """A result as the protocol carries it: bytes go back tagged, in lowercase base16."""
    return {"base16": v.hex()} if isinstance(v, bytes) else v


def call(name: str, args: list) -> object:
    """Run the named function with json-decoded arguments; a constant is a function with no arguments."""
    if name not in functions:
        raise KeyError(f"unknown function: {name}")
    f = functions[name]
    args = [_from_protocol(a) for a in args]
    return _to_protocol(f(*args) if callable(f) else f)


def main() -> None:
    """Serve requests until stdin closes."""
    for line in sys.stdin:
        request = json.loads(line)
        response = {"id": request.get("id")}
        try:
            response["result"] = call(request["call"], request["args"])
        except Exception as e:  # noqa: BLE001, this is the top gate: report anything that escaped as the response, and keep serving
            response["error"] = str(e)
        print(json.dumps(response), flush=True)


if __name__ == "__main__":
    main()

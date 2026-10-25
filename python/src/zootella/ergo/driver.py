"""The python edition's driver: one json request per line on stdin, one json response per line on stdout.

Also the dispatch table from the protocol's function names, which are the JavaScript edition's, to this edition's. A constant is a function with no arguments, and a dotted name like Size.kb is a constant in a namespace. Bytes cross as tagged json: {"base16": "616263"} is those three bytes, and {"repeat": "61", "length": 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line. The vector reader in tests uses the table too.
"""

import io
import json
import sys

from zootella.ergo import (
    BLAKE3,
    DAY_NAMES,
    HASH_PROTOCOL,
    MONTH_NAMES,
    OUTLINE_LIMITS,
    SHA1,
    SHA256,
    TAG_LENGTH,
    Bin,
    Data,
    FujiHasher,
    HashMeasurement,
    HashProtocol,
    HashUnit,
    Outline,
    OutlineLimits,
    Size,
    Span,
    Time,
    blake3,
    check_int,
    check_outline_name,
    check_string,
    check_tag,
    check_text,
    check_tick,
    cut,
    cut2,
    cut_after_last,
    cut_last,
    data_to_outline,
    data_to_quoted,
    deindent,
    has_int,
    has_outline_name,
    has_string,
    has_tag,
    has_text,
    has_tick,
    hash_file,
    hash_measure,
    hash_stream,
    hash_stripe_overlap,
    hash_text,
    iso_to_tick,
    quoted_to_data,
    say_duration,
    say_size4,
    say_tick,
    say_time_race,
    say_time_remaining,
    say_time_remaining_coarse,
    sha1,
    sha256,
    span_decode,
    span_encode,
    tag,
    text_to_outline,
    text_to_tick,
    tick_to_iso,
    tick_to_text,
    trim,
)

functions = {
    "hasInt": has_int,
    "checkInt": check_int,
    "hasString": has_string,
    "checkString": check_string,
    "hasText": has_text,
    "checkText": check_text,
    "trim": trim,
    # a named tuple crosses as an object, the way javascript's answer does
    "cut": lambda s, tag: cut(s, tag)._asdict(),
    "cutLast": lambda s, tag: cut_last(s, tag)._asdict(),
    "cut2": lambda s, tag1, tag2: cut2(s, tag1, tag2)._asdict(),
    "cutAfterLast": cut_after_last,
    "deindent": deindent,
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
    "sayDuration": say_duration,
    "sayTimeRemaining": say_time_remaining,
    "sayTimeRemainingCoarse": say_time_remaining_coarse,
    "sayTimeRace": say_time_race,
    "hasTick": has_tick,
    "checkTick": check_tick,
    "tickToText": tick_to_text,
    "textToTick": text_to_tick,
    "tickToIso": tick_to_iso,
    "isoToTick": iso_to_tick,
    "monthNames": list(MONTH_NAMES),  # a list, as json carries it
    "dayNames": list(DAY_NAMES),
    "sayTick": say_tick,
    "tagLength": TAG_LENGTH,
    "Tag": tag,
    "hasTag": has_tag,
    "checkTag": check_tag,
    "sha1": sha1,
    "sha256": sha256,
    "blake3": blake3,
    "hashText": hash_text,
    # the protocol's names for things that keep state, each answering in one call, so a vector pins them without the harness keeping an object between calls
    "Sha1": lambda pieces: _fed(SHA1(), pieces),
    "Sha256": lambda pieces: _fed(SHA256(), pieces),
    "Blake3": lambda pieces: _fed(BLAKE3(), pieces),
    "Data": lambda form, method, *args: _data_method(form, method, args),
    "Bin": lambda capacity, items, method: _bin_method(capacity, items, method),
    "hashProtocol.pieces.title": HASH_PROTOCOL.pieces.title,
    "hashProtocol.pieces.size": HASH_PROTOCOL.pieces.size,
    "hashProtocol.tips.title": HASH_PROTOCOL.tips.title,
    "hashProtocol.tips.size": HASH_PROTOCOL.tips.size,
    "hashMeasure": lambda measure: _measured(
        hash_measure(measure["file"], measure["unit"])
    ),
    "hashStripeOverlap": lambda foreground, background: [
        list(o)
        for o in hash_stripe_overlap(tuple(foreground), [tuple(b) for b in background])
    ],
    "hashFile": lambda file, protocol=None: {
        "tipHash": hash_file(
            io.BytesIO(file.array()), file.size(), _protocol(protocol)
        ).tip_hash
    },
    "hashStream": lambda pieces, protocol=None: _hashed_stream(
        pieces, _protocol(protocol)
    ),
    "FujiHasher": lambda size, pieces, protocol=None: _fed_fuji(
        FujiHasher(size, _protocol(protocol)), pieces
    ),
    "hasOutlineName": has_outline_name,
    "checkOutlineName": check_outline_name,
    "Outline": lambda o, method, *args: _outline_method(o, method, args),
    "textToOutline": lambda s, limits=None: text_to_outline(s, _outline_limits(limits)),
    "outlineLimits.size": OUTLINE_LIMITS.size,
    "outlineLimits.depth": OUTLINE_LIMITS.depth,
    "outlineLimits.nameSize": OUTLINE_LIMITS.name_size,
    "dataToOutline": lambda d, limits=None: data_to_outline(d, _outline_limits(limits)),
    "spanEncode": span_encode,
    "spanDecode": lambda data, offset: _span_answer(span_decode(data, offset)),
    "dataToQuoted": data_to_quoted,
    "quotedToData": quoted_to_data,
}


def _measured(m: HashMeasurement) -> dict:
    """What every edition measures alike: the pieces, and the stripes the tip hash reads."""
    return {
        "pieces": m.pieces,
        "complete": m.complete,
        "fragment": m.fragment,
        "stripes": [list(s) for s in m.stripes],
        "stripeSize": m.stripe_size,
    }


def _protocol(p: dict | None) -> HashProtocol:
    """A protocol from json like {"pieces": {"title": ..., "size": ...}, "tips": {...}}, or the fuji one when there's none."""
    if p is None:
        return HASH_PROTOCOL
    return HashProtocol(HashUnit(**p["pieces"]), HashUnit(**p["tips"]))


class _Boxes:
    """A stream that delivers the given pieces as its boxes, in order, each read answering no more than the rest of one piece."""

    def __init__(self, pieces: list[Data]) -> None:
        self._pieces = [p.array() for p in pieces if p.size() > 0]

    def read(self, n: int) -> bytes:
        if not self._pieces:
            return b""
        box, rest = self._pieces[0][:n], self._pieces[0][n:]
        if rest:
            self._pieces[0] = rest
        else:
            self._pieces.pop(0)
        return box


def _hashed_stream(pieces: list[Data], protocol: HashProtocol) -> dict:
    """hashStream takes the stream as a list of pieces, which arrive as its boxes in order, and answers both hashes."""
    size = sum(p.size() for p in pieces)
    status = hash_stream(_Boxes(pieces), size, protocol)
    return {"tipHash": status.tip_hash, "pieceHash": status.piece_hash}


def _fed_fuji(h: FujiHasher, pieces: list[Data]) -> dict:
    """FujiHasher takes the size it's told, the pieces to feed it in order, and a protocol, and answers both hashes, or raises when the pieces don't add up to the size."""
    for piece in pieces:
        h.update(piece)
    hashes = h.digest()
    return {"tipHash": hashes.tip_hash, "pieceHash": hashes.piece_hash}


def _mulberry_data(seed: int, length: int) -> Data:
    """{"mulberry": seed, "length": n} is n bytes from the Mulberry32 generator at that seed, wrapping at 32 bits as every edition must, so a vector can name a realistic file of any size in a line, the way the original, the JavaScript ergo grew from, makes its test files; Mulberry32 is a well-known small PRNG by Tommy Ettinger from 2018."""
    s = seed & 0xFFFFFFFF
    a = bytearray(length)
    v = 0
    for i in range(length):
        if i % 4 == 0:
            # a new 32-bit value every 4 bytes; python's ints don't wrap, so mask to 32 bits after each step the way javascript's Math.imul and >>> do
            s = (s + 0x6D2B79F5) & 0xFFFFFFFF
            t = s
            t = ((t ^ (t >> 15)) * (t | 1)) & 0xFFFFFFFF
            t ^= (t + ((t ^ (t >> 7)) * (t | 61))) & 0xFFFFFFFF
            v = (t ^ (t >> 14)) & 0xFFFFFFFF
        a[i] = (
            v >> ((i % 4) * 8)
        ) & 0xFF  # the appropriate byte from it, low byte first
    return Data(array=bytes(a))


def _fed(h: SHA1 | SHA256 | BLAKE3, pieces: list[Data]) -> Data:
    """A hasher takes one argument, a list of pieces, and answers the hash of all of them fed in order."""
    for piece in pieces:
        h.update(piece)
    return h.digest()


# the protocol spells a method the way the javascript edition does
_method_names = {"clipView": "clip_view", "clipCopy": "clip_copy"}


def _data_method(form: Data | dict, method: str, args: tuple) -> object:
    """A Data takes the form to make it from, then the name of a method and that method's arguments, like [{"text": "ABC"}, "base16"]; tagged bytes arrive already a Data, and any other form, like {"text": ...}, makes one here."""
    d = form if isinstance(form, Data) else Data(**form)
    return getattr(d, _method_names.get(method, method))(*args)


def _bin_method(capacity: int, items: list, method: str) -> object:
    """A Bin takes its capacity, a list of what to add, each a byte or a Data, and the name of a method to answer with."""
    b = Bin(capacity)
    for item in items:
        b.add(item)
    return getattr(b, method)()


def _outline_method(o: Outline | str | None, method: str, args: tuple) -> object:
    """An Outline takes the outline to start from, or a name to make one with, then the name of a method and its arguments, like [outline, "text"] or [outline, "list", "jar"]; a method that answers nothing, like a setter or sort, answers the outline after it."""
    if not isinstance(o, Outline):
        o = Outline(
            o
        )  # an outline arrives already made from its json; a name, or null for none, makes one here
    # the protocol's name and value get with no argument and set with one, the way javascript's do, where this edition has a setter of each
    if method in ("name", "value") and args:
        method = "set_" + method
    answer = getattr(o, method)(*args)
    if answer is None:
        return o
    return answer


def _outline_limits(v: dict | None) -> OutlineLimits:
    """Limits from json like {"size": 64, "nameSize": 8}, any left out keeping their default, as javascript's spread does, or the defaults when there are none."""
    if v is None:
        return OUTLINE_LIMITS
    names = {"size": "size", "depth": "depth", "nameSize": "name_size"}
    return OutlineLimits(**{names[k]: n for k, n in v.items()})


def _span_answer(s: Span) -> dict:
    """What spanDecode answers, as javascript's object spells it."""
    return {"value": s.value, "bytesRead": s.bytes_read}


def _from_protocol(v: object) -> object:
    """An argument as this edition takes it: tagged bytes become a Data, a list is decoded all through, and anything else passes as it is."""
    if isinstance(v, list):
        return [_from_protocol(x) for x in v]
    if isinstance(v, dict) and "base16" in v:
        # ergo's own base16 reader, which takes exactly what its writer makes, lowercase and nothing else
        return Data(base16=v["base16"])
    if isinstance(v, dict) and "mulberry" in v:
        return _mulberry_data(v["mulberry"], v["length"])
    if isinstance(v, dict) and "contents" in v:
        # an outline is {name, value, contents}, its value as tagged bytes and its contents as outlines, in the order the outline holds them
        o = Outline(v["name"], _from_protocol(v["value"]))
        for c in v["contents"]:
            o.add(_from_protocol(c))
        return o
    if isinstance(v, dict) and "repeat" in v:
        unit, n = Data(base16=v["repeat"]).array(), v.get("length")
        # a unit of at least one byte, repeated to a count within the integer ceiling, the same rule in every edition; a bool is an int in python, and never a count
        if (
            not unit
            or not isinstance(n, int)
            or isinstance(n, bool)
            or not 0 <= n <= 2**53 - 1
        ):
            raise ValueError(f"not bytes: {v!r}")
        return Data(array=(unit * (n // len(unit) + 1))[:n])
    return v


def _to_protocol(v: object) -> object:
    """A result as the protocol carries it: a Data goes back tagged, as its base16, and an Outline as {name, value, contents}, in its order, not sorted, wherever it is in the answer."""
    if isinstance(v, Data):
        return {"base16": v.base16()}
    if isinstance(v, Outline):
        return {
            "name": v.name(),
            "value": _to_protocol(v.value()),
            "contents": [_to_protocol(v.get(i)) for i in range(v.length())],
        }
    if isinstance(v, dict):
        return {k: _to_protocol(x) for k, x in v.items()}
    if isinstance(v, list):
        return [_to_protocol(x) for x in v]
    return v


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
        # outside the top gate: a line the driver can't read is a bug in the harness, not a call to refuse, so it stops loudly
        request = json.loads(line)
        response = {"id": request.get("id")}
        try:
            response["result"] = call(request["call"], request["args"])
        except Exception as e:  # noqa: BLE001, this is the top gate: report anything that escaped as the response, and keep serving
            response["error"] = str(e)
        print(json.dumps(response), flush=True)


if __name__ == "__main__":
    main()

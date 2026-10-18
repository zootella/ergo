"""The python edition's driver: one json request per line on stdin, one json response per line on stdout.

Also the dispatch table from the protocol's function names, which are the JavaScript edition's, to this edition's. A constant is a function with no arguments, and a dotted name like Size.kb is a constant in a namespace. The vector reader in tests uses the table too.
"""

import json
import sys

from zootella.ergo import (
    TAG_LENGTH,
    Size,
    Time,
    check_tag,
    check_tick,
    has_tag,
    has_tick,
    iso_to_tick,
    say_size4,
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
}


def call(name: str, args: list) -> object:
    """Run the named function with json-decoded arguments; a constant is a function with no arguments."""
    if name not in functions:
        raise KeyError(f"unknown function: {name}")
    f = functions[name]
    return f(*args) if callable(f) else f


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

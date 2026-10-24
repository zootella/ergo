"""Outline's own test in this edition, beside the shared vectors: every walk keeps a stack of its own, so an outline goes as deep as memory allows."""

from zootella.ergo import Data, Outline, data_to_outline


def test_outline_goes_deep() -> None:
    """Two chains ten thousand levels down write, read back, sort, and compare, ten times past the thousand frames Python gives a walk that calls itself."""

    # the text form isn't here, since a line's indent grows with its depth, so the text of an outline this deep would be a hundred million spaces
    def chain(bottom: str) -> Outline:
        top = Outline("a")
        o = top
        for _ in range(10000):
            o = o.m("a")
        o.set_value(Data(base16=bottom))
        return top

    def bottom_of(o: Outline) -> Outline:
        while o.length() > 0:
            o = o.get(0)
        return o

    o = Outline("x")
    o.add(chain("01"))
    # the same all the way down but the value at the bottom, and in the wrong order, so putting them in order compares ten thousand levels deep
    o.add(chain("00"))
    d = o.data()
    back = data_to_outline(d)
    assert back.data().base16() == d.base16()
    assert (
        bottom_of(back.get(0)).value().base16() == "00"
    )  # the bytes came out in order
    o.sort()
    assert (
        bottom_of(o.get(0)).value().base16() == "00"
    )  # and sort put them in order, too

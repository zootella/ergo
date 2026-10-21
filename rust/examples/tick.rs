//! Tick: write a moment two ways, and read each back exactly, and say one for a person. Run with cargo run --example tick

use zootella_ergo::{has_tick, iso_to_tick, say_tick, text_to_tick, tick_to_iso, tick_to_text};

fn main() {
    // #region tick
    let t = 1_344_230_277_300; // a tick: milliseconds since 1970; this one is the Curiosity landing
    let text = tick_to_text(t); // "2012aug6.0517.57300", as short as the moment allows
    let iso = tick_to_iso(t); // "2012-08-06T05:17:57.300Z", always 24 characters
    let same = text_to_tick(&text) == Ok(t) && iso_to_tick(&iso) == Ok(t); // each form reads back to exactly t, and refuses any text its writer wouldn't produce
    let asked = has_tick(-1); // false: ask about a number from outside, like a clock or a file's modified time, before handing it to a function that takes a tick, which panics on one that isn't
    // #endregion tick
    println!("{text} {iso} {same} {asked}");

    // #region saytick
    let t = 1_344_230_277_300; // the Curiosity landing
    let page = say_tick(t, "Dddd, Mmmm D, YYYY"); // "Monday, August 6, 2012": uppercase tags are the calendar
    let log = say_tick(t, "YYYY-MM-DD hh24:mm:ss.ttt"); // "2012-08-06 05:17:57.300": lowercase tags are the clock
    let clock = say_tick(t, "h12:mm AA"); // "5:17 AM"
    // #endregion saytick
    println!("{page} {log} {clock}");
}

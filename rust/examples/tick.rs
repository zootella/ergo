//! Tick: write a moment two ways, and read each back exactly. Run with cargo run --example tick

use zootella_ergo::{iso_to_tick, text_to_tick, tick_to_iso, tick_to_text};

fn main() {
    // #region tick
    let t = 1_344_230_277_300; // a tick: milliseconds since 1970; this one is the Curiosity landing
    let text = tick_to_text(t); // "2012aug6.0517.57300", as short as the moment allows
    let iso = tick_to_iso(t); // "2012-08-06T05:17:57.300Z", always 24 characters
    let same = text_to_tick(&text) == Ok(t) && iso_to_tick(&iso) == Ok(t); // each form reads back to exactly t, and refuses any text its writer wouldn't produce
    // #endregion tick
    println!("{text} {iso} {same}");
}

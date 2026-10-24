//! Text: trim it, ask whether it holds anything, cut it at a tag, and take the indent off a multiline string. Run with cargo run --example text

use zootella_ergo::{cut, deindent, has_text, trim};

fn main() {
    // #region text
    let name = trim("  Alice  "); // "Alice": whitespace off both ends, by Unicode's list in every edition
    let blank = has_text("   "); // false: nothing there but whitespace
    let c = cut("key=value", "="); // found, with "key" before the tag and "value" after it
    let note = deindent(
        "
        line one
          line two
    ",
    ); // "line one\n  line two\n": the first line's indent comes off every line
    // #endregion text
    println!(
        "{name} {blank} {} {} {} {note:?}",
        c.found, c.before, c.after
    );
}

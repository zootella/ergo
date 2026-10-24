//! Outline: read one from text, navigate it, build on it, and write it back as text and as bytes. Run with cargo run --example outline

use zootella_ergo::{Data, data_to_outline, deindent, text_to_outline};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #region outline
    let mut o = text_to_outline(&deindent(
        r#"
        orchard:
          tree:"apple"
          basket:
            color:"red"  # a comment, which the reader skips
        "#,
    ))?; // read from text written by hand, contents in any order
    let tree = o.n("tree").value().text()?; // "apple": n navigates to a name
    o.m("basket").m("count").set_value(Data::from_text("12")); // m makes as it navigates, so a chain builds as it goes
    let text = o.text(); // written back sorted, at every level, two spaces an indent
    let back = data_to_outline(&o.data())?.text() == text; // true: the bytes, the canonical form, read back to the same outline
    // #endregion outline
    println!("{tree} {back}\n{text}");
    Ok(())
}

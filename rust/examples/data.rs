//! Data and Bin: bytes in every form, and bytes assembled to a set size. Run with cargo run --example data

use zootella_ergo::{Bin, Data};

fn main() -> Result<(), zootella_ergo::DataError> {
    // #region data
    let d = Data::from_text("hello"); // bytes, from text
    let forms = (d.base16(), d.base32(), d.base62(), d.base64()); // out in any form: 68656c6c6f, NBSWY3DP, Q6LhR6z, aGVsbG8=
    let back = Data::from_base62("Q6LhR6z")?.text()?; // and back from any, each read exactly: "hello"
    let part = d.clip_view(1, 3).text()?; // part of it, without copying: "el"
    let mut b = Bin::new(6); // a set number of bytes, filled a byte or a Data at a time
    b.add(&d);
    b.add_byte(b'!');
    let whole = b.data().text()?; // "hello!"
    // #endregion data
    println!("{forms:?} {back} {part} {whole}");
    Ok(())
}

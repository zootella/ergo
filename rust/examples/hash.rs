//! Hash: SHA-1, SHA-256, and BLAKE3, all at once or a piece at a time. Run with cargo run --example hash

use std::io::Read;
use zootella_ergo::{Blake3, Sha1, Sha256, sha256, size};

fn main() -> std::io::Result<()> {
    // #region hash
    let d = sha256(b"hello"); // all at once, for bytes already in memory: 32 bytes
    let mut file = &b"hello"[..]; // or File::open(path)?, a file of any size
    let (mut h1, mut h256, mut hb3) = (Sha1::new(), Sha256::new(), Blake3::new()); // or a piece at a time, all three from one read of the file
    let mut piece = vec![0; 64 * size::KB as usize];
    loop {
        let n = file.read(&mut piece)?;
        if n == 0 {
            break;
        }
        h1.update(&piece[..n]);
        h256.update(&piece[..n]);
        hb3.update(&piece[..n]);
    }
    let (d1, d256, db3) = (h1.finalize(), h256.finalize(), hb3.finalize());
    // #endregion hash
    for h in [&d[..], &d1[..], &d256[..], &db3[..]] {
        println!("{}", hex(h));
    }
    Ok(())
}

/// Lowercase base16, the form the world shows a hash in, which the standard library leaves to us.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

//! The fuji hashes: a file's tip hash from a few small reads, and its piece hash from one read of all of it. Run with cargo run --example fuji

use zootella_ergo::{hash_file, hash_stream};

fn main() -> std::io::Result<()> {
    // #region fuji
    let mut file = std::io::Cursor::new(b"hello"); // or File::open(path)?, with its size from metadata()?.len()
    let tip = hash_file(&mut file, 5)?.tip_hash; // the tip hash, from a few small reads
    file.set_position(0);
    let piece = hash_stream(&mut file, 5, |_| {})?.piece_hash; // the piece hash, from one read of all of it, which gives the tip hash too
    // #endregion fuji
    println!(
        "{} {}",
        tip.expect("a finished hash").base32(),
        piece.expect("a finished hash").base32()
    );
    Ok(())
}

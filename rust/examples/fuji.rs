//! The fuji hashes: a file's tip hash from a few small reads, and its piece hash from one read of all of it. Run with cargo run --example fuji

use zootella_ergo::{Data, FujiHasher, hash_file, hash_stream};

fn main() -> std::io::Result<()> {
    // #region fuji
    let mut file = std::io::Cursor::new(b"hello"); // or File::open(path)?, with its size from metadata()?.len()
    let tip = hash_file(&mut file, 5)?.tip_hash; // the tip hash, from a few small reads
    file.set_position(0);
    let piece = hash_stream(&mut file, 5, |_| {})?.piece_hash; // the piece hash, from one read of all of it, which gives the tip hash too

    let mut hasher = FujiHasher::new(5); // or feed one the pieces as they arrive, like the chunks of an upload
    hasher.update(&Data::from_text("he"));
    hasher.update(&Data::from_text("llo"));
    let both = hasher.finalize()?; // both hashes, the same as from the stream
    // #endregion fuji
    assert_eq!(Some(&both.tip_hash), tip.as_ref());
    assert_eq!(Some(&both.piece_hash), piece.as_ref());
    println!(
        "{} {}",
        tip.expect("a finished hash").base32(),
        piece.expect("a finished hash").base32()
    );
    Ok(())
}

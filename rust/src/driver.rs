//! The dispatch table from the protocol's function names, which are the JavaScript edition's, to this edition's functions, with json arguments and results. The vector reader in tests and the driver binary both use it; it is not part of the library's api.

use serde_json::{Value, json};

/// Run the named function with json-decoded arguments; a constant is a function with no arguments, and a dotted name like "Size.kb" is a constant in a namespace. A panic comes back as an error rather than unwinding into the caller, and so does an Err from a function that returns one, so the harness sees both as the call being refused.
pub fn dispatch(call: &str, args: &[Value]) -> Result<Value, String> {
    std::panic::catch_unwind(|| run(call, args)).unwrap_or_else(|p| Err(panic_message(&p)))
}

fn run(call: &str, args: &[Value]) -> Result<Value, String> {
    match call {
        "Size.b" => Ok(json!(crate::size::B)),
        "Size.kb" => Ok(json!(crate::size::KB)),
        "Size.mb" => Ok(json!(crate::size::MB)),
        "Size.gb" => Ok(json!(crate::size::GB)),
        "Size.tb" => Ok(json!(crate::size::TB)),
        "Size.pb" => Ok(json!(crate::size::PB)),
        "saySize4" => Ok(json!(crate::say_size4(count(args, 0)?))),
        "Time.millisecond" => Ok(json!(crate::time::MILLISECOND)),
        "Time.second" => Ok(json!(crate::time::SECOND)),
        "Time.minute" => Ok(json!(crate::time::MINUTE)),
        "Time.hour" => Ok(json!(crate::time::HOUR)),
        "Time.day" => Ok(json!(crate::time::DAY)),
        "Time.week" => Ok(json!(crate::time::WEEK)),
        "Time.year" => Ok(json!(crate::time::YEAR)),
        "Time.month" => Ok(json!(crate::time::MONTH)),
        "hasTick" => Ok(json!(crate::has_tick(integer(args, 0)?))),
        "checkTick" => Ok(json!(crate::check_tick(integer(args, 0)?))),
        "tickToText" => Ok(json!(crate::tick_to_text(integer(args, 0)?))),
        "textToTick" => refused(crate::text_to_tick(text(args, 0)?)),
        "tickToIso" => Ok(json!(crate::tick_to_iso(integer(args, 0)?))),
        "isoToTick" => refused(crate::iso_to_tick(text(args, 0)?)),
        "tagLength" => Ok(json!(crate::TAG_LENGTH)),
        "Tag" => Ok(json!(crate::tag())),
        "hasTag" => Ok(json!(crate::has_tag(text(args, 0)?))),
        "checkTag" => Ok(json!(crate::check_tag(text(args, 0)?))),
        "sha1" => Ok(base16(&crate::sha1(&bytes(args, 0)?))),
        "sha256" => Ok(base16(&crate::sha256(&bytes(args, 0)?))),
        "blake3" => Ok(base16(&crate::blake3(&bytes(args, 0)?))),
        "hashText" => Ok(json!(crate::hash_text(text(args, 0)?))),
        "Sha1" => Ok(base16(
            &fed(crate::Sha1::new(), crate::Sha1::update, &pieces(args, 0)?).finalize(),
        )),
        "Sha256" => Ok(base16(
            &fed(
                crate::Sha256::new(),
                crate::Sha256::update,
                &pieces(args, 0)?,
            )
            .finalize(),
        )),
        "Blake3" => Ok(base16(
            &fed(
                crate::Blake3::new(),
                crate::Blake3::update,
                &pieces(args, 0)?,
            )
            .finalize(),
        )),
        "Data" => data_method(args),
        "Bin" => bin_method(args),
        "hashProtocol.pieces.title" => Ok(json!(crate::HashProtocol::default().pieces.title)),
        "hashProtocol.pieces.size" => Ok(json!(crate::HashProtocol::default().pieces.size)),
        "hashProtocol.tips.title" => Ok(json!(crate::HashProtocol::default().tips.title)),
        "hashProtocol.tips.size" => Ok(json!(crate::HashProtocol::default().tips.size)),
        "hashMeasure" => {
            let m = args.first().ok_or("argument 0 must be a measure")?;
            let field = |k: &str| m[k].as_u64().ok_or_else(|| format!("{k} must be a count"));
            Ok(measured(&crate::hash_measure(
                field("file")?,
                field("unit")?,
            )))
        }
        "hashStripeOverlap" => {
            let foreground = stripe(args.first().ok_or("argument 0 must be a stripe")?)?;
            let background = args
                .get(1)
                .and_then(Value::as_array)
                .ok_or("argument 1 must be a list of stripes")?
                .iter()
                .map(stripe)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(
                crate::hash_stripe_overlap(foreground, &background)
                    .iter()
                    .map(|&(a, b)| json!([a, b]))
                    .collect::<Vec<_>>()
            ))
        }
        "hashFile" => {
            let file = bytes(args, 0)?;
            let status = protocol(args.get(1))?
                .hash_file(&mut std::io::Cursor::new(file.array()), file.size() as u64)
                .map_err(|e| e.to_string())?;
            Ok(json!({"tipHash": base16(&status.tip_hash.expect("a finished hash"))}))
        }
        "hashStream" => {
            let list = pieces(args, 0)?;
            let size = list.iter().map(|p| p.size() as u64).sum();
            let status = protocol(args.get(1))?
                .hash_stream(
                    &mut Boxes {
                        pieces: list,
                        at: 0,
                        into: 0,
                    },
                    size,
                    |_| {},
                )
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "tipHash": base16(&status.tip_hash.expect("a finished hash")),
                "pieceHash": base16(&status.piece_hash.expect("a finished hash")),
            }))
        }
        "FujiHasher" => {
            let h = protocol(args.get(2))?.fuji_hasher(count(args, 0)?);
            let hashes = fed(h, crate::FujiHasher::update, &pieces(args, 1)?)
                .finalize()
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "tipHash": base16(&hashes.tip_hash),
                "pieceHash": base16(&hashes.piece_hash),
            }))
        }
        _ => Err(format!("unknown function: {call}")),
    }
}

/// A reader's own Result as the protocol's: the tick, or the error's message as the refusal.
fn refused<E: std::fmt::Display>(r: Result<i64, E>) -> Result<Value, String> {
    r.map(|t| json!(t)).map_err(|e| e.to_string())
}

/// Argument i as a count, zero or more, or an error naming the argument.
fn count(args: &[Value], i: usize) -> Result<u64, String> {
    args.get(i)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("argument {i} must be a count"))
}

/// Argument i as an integer, which may be negative, or an error naming the argument.
fn integer(args: &[Value], i: usize) -> Result<i64, String> {
    args.get(i)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("argument {i} must be an integer"))
}

/// Argument i as text, or an error naming the argument.
fn text(args: &[Value], i: usize) -> Result<&str, String> {
    args.get(i)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("argument {i} must be text"))
}

/// A hasher in the protocol takes one argument, a list of pieces, and answers the hash of all of them fed in order, so a vector pins the piece-at-a-time form without the harness keeping an object between calls. Each piece goes through the hasher's own update, empty ones included, which io::Write's write_all would skip.
fn fed<H>(mut h: H, update: fn(&mut H, &crate::Data), pieces: &[crate::Data]) -> H {
    for piece in pieces {
        update(&mut h, piece);
    }
    h
}

/// A Data in the protocol takes the form to make it from, then the name of a method and that method's arguments, like [{"text": "ABC"}, "base16"] or [{"base16": "414243"}, "clipView", 1, 3]. Tagged bytes are already a Data; any other form, like {"text": ...}, makes one here.
fn data_method(args: &[Value]) -> Result<Value, String> {
    let form = args.first().ok_or("argument 0 must be a form")?;
    let d = match tagged(form) {
        Some(d) => d?,
        None => made(form)?,
    };
    let method = text(args, 1)?;
    let index = |i: usize| -> Result<usize, String> {
        usize::try_from(count(args, i)?).map_err(|e| e.to_string())
    };
    match method {
        "size" => Ok(json!(d.size())),
        "text" => d.text().map(|t| json!(t)).map_err(|e| e.to_string()),
        "base16" => Ok(json!(d.base16())),
        "base32" => Ok(json!(d.base32())),
        "base62" => Ok(json!(d.base62())),
        "base64" => Ok(json!(d.base64())),
        "get" => Ok(json!(d.get(index(2)?))),
        "clipView" => Ok(base16(&d.clip_view(index(2)?, index(3)?))),
        "clipCopy" => Ok(base16(&d.clip_copy(index(2)?, index(3)?))),
        _ => Err(format!("unknown method of Data: {method}")),
    }
}

/// A Data made from a form that isn't tagged bytes: {"text": ...}, {"base32": ...}, {"base62": ...}, {"base64": ...}, or {"random": n}.
fn made(form: &Value) -> Result<crate::Data, String> {
    let as_text = |key: &str| form[key].as_str();
    let refused = |e: crate::DataError| e.to_string();
    if let Some(s) = as_text("text") {
        Ok(crate::Data::from_text(s))
    } else if let Some(s) = as_text("base32") {
        crate::Data::from_base32(s).map_err(refused)
    } else if let Some(s) = as_text("base62") {
        crate::Data::from_base62(s).map_err(refused)
    } else if let Some(s) = as_text("base64") {
        crate::Data::from_base64(s).map_err(refused)
    } else if let Some(n) = form["random"].as_u64() {
        Ok(crate::Data::random(
            usize::try_from(n).map_err(|e| e.to_string())?,
        ))
    } else {
        Err(format!("not a form a Data is made from: {form}"))
    }
}

/// A Bin in the protocol takes its capacity, a list of what to add, each a byte or a Data, and the name of a method to answer with.
fn bin_method(args: &[Value]) -> Result<Value, String> {
    let mut b = crate::Bin::new(usize::try_from(count(args, 0)?).map_err(|e| e.to_string())?);
    let items = args
        .get(1)
        .and_then(Value::as_array)
        .ok_or("argument 1 must be a list")?;
    for item in items {
        if let Some(d) = tagged(item) {
            b.add(&d?);
        } else {
            let byte = item.as_u64().and_then(|n| u8::try_from(n).ok());
            b.add_byte(byte.ok_or_else(|| format!("not a byte: {item}"))?);
        }
    }
    match text(args, 2)? {
        "size" => Ok(json!(b.size())),
        "capacity" => Ok(json!(b.capacity())),
        "data" => Ok(base16(&b.data())),
        method => Err(format!("unknown method of Bin: {method}")),
    }
}

/// What every edition measures alike: the pieces, and the stripes the tip hash reads.
fn measured(m: &crate::HashMeasurement) -> Value {
    json!({
        "pieces": m.pieces,
        "complete": m.complete,
        "fragment": m.fragment,
        "stripes": m.stripes.iter().map(|&(a, b)| json!([a, b])).collect::<Vec<_>>(),
        "stripeSize": m.stripe_size,
    })
}

/// A stripe from json, [start, end].
fn stripe(v: &Value) -> Result<(u64, u64), String> {
    match (v[0].as_u64(), v[1].as_u64()) {
        (Some(a), Some(b)) => Ok((a, b)),
        _ => Err(format!("not a stripe: {v}")),
    }
}

/// A protocol from json like {"pieces": {"title": ..., "size": ...}, "tips": {...}}, or the fuji one when there's none.
fn protocol(v: Option<&Value>) -> Result<crate::HashProtocol, String> {
    let Some(v) = v else {
        return Ok(crate::HashProtocol::default());
    };
    let unit = |u: &Value| -> Result<crate::HashUnit, String> {
        Ok(crate::HashUnit {
            title: u["title"]
                .as_str()
                .ok_or("a unit needs a title")?
                .to_string(),
            size: u["size"].as_u64().ok_or("a unit needs a size")?,
        })
    };
    Ok(crate::HashProtocol {
        pieces: unit(&v["pieces"])?,
        tips: unit(&v["tips"])?,
    })
}

/// A stream that delivers the given pieces as its boxes, in order, each read answering no more than the rest of one piece.
struct Boxes {
    pieces: Vec<crate::Data>,
    at: usize,   // which piece is next
    into: usize, // how far into it
}

impl std::io::Read for Boxes {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        while self.at < self.pieces.len() && self.into == self.pieces[self.at].size() {
            self.at += 1; // past a piece that's done, or empty
            self.into = 0;
        }
        let Some(piece) = self.pieces.get(self.at) else {
            return Ok(0); // the end of the stream
        };
        let n = (piece.size() - self.into).min(buffer.len());
        buffer[..n].copy_from_slice(&piece.array()[self.into..self.into + n]);
        self.into += n;
        Ok(n)
    }
}

/// {"mulberry": seed, "length": n} is n bytes from icarus's Mulberry32 generator at that seed, wrapping at 32 bits as every edition must, so a vector can name a realistic file of any size in a line; Mulberry32 is a well-known small PRNG by Tommy Ettinger from 2018.
fn mulberry_data(seed: u32, length: usize) -> crate::Data {
    let mut s = seed;
    let mut a = vec![0u8; length];
    let mut v = 0u32;
    for (i, byte) in a.iter_mut().enumerate() {
        if i % 4 == 0 {
            // a new 32-bit value every 4 bytes
            s = s.wrapping_add(0x6d2b_79f5);
            let mut t = s;
            t = (t ^ (t >> 15)).wrapping_mul(t | 1);
            t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
            v = t ^ (t >> 14);
        }
        *byte = (v >> ((i % 4) * 8)) as u8; // the appropriate byte from it, low byte first
    }
    crate::Data::from(a)
}

/// Argument i as a Data, from either tagged form, or an error naming the argument.
fn bytes(args: &[Value], i: usize) -> Result<crate::Data, String> {
    match args.get(i).and_then(tagged) {
        Some(d) => d,
        None => Err(format!("argument {i} must be bytes")),
    }
}

/// Argument i as a list of pieces, each a Data, or an error naming the argument.
fn pieces(args: &[Value], i: usize) -> Result<Vec<crate::Data>, String> {
    let list = args
        .get(i)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("argument {i} must be a list of pieces"))?;
    list.iter()
        .map(|v| tagged(v).unwrap_or_else(|| Err(format!("piece of argument {i} must be bytes"))))
        .collect()
}

/// Bytes cross the protocol as tagged json, which becomes a Data: {"base16": "616263"} is those three bytes, and {"repeat": "61", "length": 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line. None if v is neither form, and an error if it's one of them but malformed.
fn tagged(v: &Value) -> Option<Result<crate::Data, String>> {
    if let Some(s) = v["base16"].as_str() {
        return Some(crate::Data::from_base16(s).map_err(|e| e.to_string())); // ergo's own base16 reader, which takes exactly what its writer makes, lowercase and nothing else
    }
    if let (Some(seed), Some(length)) = (v["mulberry"].as_u64(), v["length"].as_u64()) {
        return Some(match (u32::try_from(seed), usize::try_from(length)) {
            (Ok(seed), Ok(length)) => Ok(mulberry_data(seed, length)),
            _ => Err(format!("not bytes: {v}")),
        });
    }
    let unit = v["repeat"].as_str()?;
    Some((|| {
        let unit = crate::Data::from_base16(unit).map_err(|e| e.to_string())?;
        let length = v["length"].as_u64().filter(|&n| n < 1 << 53); // a count within the integer ceiling
        let length = length.and_then(|n| usize::try_from(n).ok());
        match length {
            Some(n) if !unit.array().is_empty() => Ok(crate::Data::from(
                unit.array()
                    .iter()
                    .copied()
                    .cycle()
                    .take(n)
                    .collect::<Vec<u8>>(),
            )), // a unit of at least one byte, the same rule in every edition
            _ => Err(format!("not bytes: {v}")),
        }
    })())
}

/// A result that is a Data, tagged the first way, as its base16.
fn base16(d: &crate::Data) -> Value {
    json!({"base16": d.base16()})
}

/// The message a panic carried, which is a String from a formatted assert or a &str from a literal.
fn panic_message(p: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else {
        "panic".to_string()
    }
}

/// Stop the default panic hook from printing to stderr, since a panic here is an expected answer, not a crash.
pub fn quiet_panics() {
    std::panic::set_hook(Box::new(|_| {}));
}

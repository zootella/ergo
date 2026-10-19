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
fn fed<H>(mut h: H, update: fn(&mut H, &[u8]), pieces: &[Vec<u8>]) -> H {
    for piece in pieces {
        update(&mut h, piece);
    }
    h
}

/// Argument i as bytes, from either tagged form, or an error naming the argument.
fn bytes(args: &[Value], i: usize) -> Result<Vec<u8>, String> {
    args.get(i)
        .and_then(tagged)
        .ok_or_else(|| format!("argument {i} must be bytes"))
}

/// Argument i as a list of pieces of bytes, or an error naming the argument.
fn pieces(args: &[Value], i: usize) -> Result<Vec<Vec<u8>>, String> {
    args.get(i)
        .and_then(Value::as_array)
        .and_then(|a| a.iter().map(tagged).collect())
        .ok_or_else(|| format!("argument {i} must be a list of pieces of bytes"))
}

/// Bytes cross the protocol as tagged json: {"base16": "616263"} is those three bytes, and {"repeat": "61", "length": 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line. None if v is neither.
fn tagged(v: &Value) -> Option<Vec<u8>> {
    if let Some(s) = v["base16"].as_str() {
        return from_base16(s);
    }
    let unit = from_base16(v["repeat"].as_str()?)?;
    let length = v["length"].as_u64().filter(|&n| n < 1 << 53)?; // a count within the integer ceiling
    if unit.is_empty() {
        return None; // a unit of at least one byte, the same rule in every edition
    }
    Some(
        unit.iter()
            .copied()
            .cycle()
            .take(usize::try_from(length).ok()?)
            .collect(),
    )
}

/// Bytes from lowercase or uppercase base16, or None if s has an odd length or a character that isn't a digit of base16.
fn from_base16(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None; // digits only, since from_str_radix would also take a leading plus sign, and ascii, so byte positions below are character positions
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// A result that is bytes, tagged the first way, in lowercase base16.
fn base16(b: &[u8]) -> Value {
    json!({"base16": b.iter().map(|x| format!("{x:02x}")).collect::<String>()})
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

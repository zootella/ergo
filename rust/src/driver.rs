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
        "tickToText" => Ok(json!(crate::tick_to_text(integer(args, 0)?))),
        "textToTick" => refused(crate::text_to_tick(text(args, 0)?)),
        "tickToIso" => Ok(json!(crate::tick_to_iso(integer(args, 0)?))),
        "isoToTick" => refused(crate::iso_to_tick(text(args, 0)?)),
        "tagLength" => Ok(json!(crate::TAG_LENGTH)),
        "Tag" => Ok(json!(crate::tag())),
        "hasTag" => Ok(json!(crate::has_tag(text(args, 0)?))),
        "checkTag" => Ok(json!(crate::check_tag(text(args, 0)?))),
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

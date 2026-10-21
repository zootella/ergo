//! Time: name a span of milliseconds, and say one for a person. Run with cargo run --example time

use zootella_ergo::{say_duration, say_time_race, say_time_remaining, time};

fn main() {
    // #region time
    let deadline = 1_700_000_000_000 + 2 * time::HOUR; // two hours after a tick, which counts milliseconds since 1970
    // #endregion time
    println!("{deadline}");

    // #region sayduration
    let took = say_duration(2 * time::HOUR + 5 * time::MINUTE, "# hours # minutes"); // "2 hours 5 minutes": the template is the sentence, with # where each number goes
    let ago = say_duration(3 * time::DAY, "about # days ago"); // "about 3 days ago"
    let left = say_time_remaining(9 * time::MINUTE + 59 * time::SECOND); // "9m 59s": one or two units, whichever matter at the size
    let race = say_time_race(5 * time::MINUTE + 15 * time::SECOND + 223); // "5'15\"223": to the millisecond, the way a race is timed
    // #endregion sayduration
    println!("{took} | {ago} | {left} | {race}");
}

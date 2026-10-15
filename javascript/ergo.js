//ergo, short for ergonomic: simple helper functions, the same in javascript, python, rust, and go

//      _
//  ___(_)_______
// / __| |_  / _ \
// \__ \ |/ /  __/
// |___/_/___\___|
//

//numbers of bytes in the binary units that memory and storage are measured in, 1024 to each rather than 1000, so 64 * Size.kb names what a bare 65536 would hide
//a frozen literal of plain numbers, marked pure, so every bundler drops it from a page that imports none of these; construction.md has the rule, and the shared vectors pin each value
export const Size = /*#__PURE__*/ Object.freeze({
  b: 1, //one byte, the unit
  kb: 1024, //kibibyte
  mb: 1048576, //mebibyte, 1024 kibibytes
  gb: 1073741824, //gibibyte
  tb: 1099511627776, //tebibyte
  pb: 1125899906842624, //pebibyte, and the last: 2^50, still a safe integer in every edition
})

//each 1024 of the one before, labeled the everyday way; bytes come before these, and are the one unit that pluralizes
const sizeUnits = [' KB', ' MB', ' GB', ' TB', ' PB', ' EB', ' ZB', ' YB']

//say a count of bytes in four digits or less with a unit, like "1 byte", "2 bytes", or "117 MB"; n is zero or more, and the count rounds down
export function saySize4(n) {
  if (n <= 9999) return `${n} byte${n == 1 ? '' : 's'}` //at most four digits of bytes, so say bytes; singular only for exactly one
  let w = n //whole units of the unit we're counting in
  for (let unit of sizeUnits) {
    w = Math.floor(w / Size.kb) //count in the next unit up
    if (w <= 9999) return w + unit //fits in four digits, so this is the unit to say it in
  }
  return n + '' //past yottabytes, which no integer within the ceiling reaches; say the number bare
}

//  _   _
// | |_(_)_ __ ___   ___
// | __| | '_ ` _ \ / _ \
// | |_| | | | | | |  __/
//  \__|_|_| |_| |_|\___|
//

//spans of time in milliseconds, the unit of a tick, so Date.now() + 2 * Time.hour is a deadline two hours out
export const Time = /*#__PURE__*/ Object.freeze({
  millisecond: 1, //the unit; a tick counts milliseconds since 1970
  second: 1000,
  minute: 60000, //60 seconds
  hour: 3600000, //60 minutes
  day: 86400000, //24 hours
  week: 604800000, //7 days
  year: 31557600000, //365 and a quarter days, the average over the leap cycle, which is exact in milliseconds
  month: 2629800000, //a twelfth of that year, also exact
})

//  _   _      _
// | |_(_) ___| | __
// | __| |/ __| |/ /
// | |_| | (__|   <
//  \__|_|\___|_|\_\
//

/*
A tick is an integer count of milliseconds since the start of 1970, the way Date.now() counts, and ergo keeps every tick between zero and the last millisecond of 9999. Two text forms carry a tick exactly. The iso form is what toISOString writes, like "2012-08-06T05:17:57.300Z": always 24 characters, always milliseconds, always Z. The short form is like "2012aug6.0517.57300": the year, month, and day, then a dot and the clock, then a dot and the milliseconds into the minute, written only as deep as the moment needs, so the start of 1990 is just "1990" and the longest text is 20 characters.

Each form reads back to exactly the tick that wrote it, and reads nothing else. A reader accepts the text its writer produces and refuses everything else, from a missing millisecond digit to "1990feb1" where the writer would say "1990feb". The last check in each reader is the round trip itself: parse the text, write the tick, and require the same text. That's what keeps every text one to one with its tick.
*/

const lastTick = 253402300799999 //the last millisecond of 9999, which keeps a year at four digits in every edition
const months = [
  'jan',
  'feb',
  'mar',
  'apr',
  'may',
  'jun',
  'jul',
  'aug',
  'sep',
  'oct',
  'nov',
  'dec',
]
const isoPattern =
  /^([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})\.([0-9]{3})Z$/ //the one shape of the iso form

//a guard: pass a tick through, or throw because the caller handed us something that isn't one
function _checkTick(t) {
  if (!Number.isSafeInteger(t) || t < 0 || t > lastTick)
    throw new Error(`not a tick: ${t}`)
  return t
}
//true if s is made of numerals 0 through 9 and nothing else
function _isNumerals(s) {
  return /^[0-9]+$/.test(s)
}

//write a tick in the iso form, like "2012-08-06T05:17:57.300Z"
export function tickToIso(t) {
  return new Date(_checkTick(t)).toISOString() //the platform writes exactly the form we read: a four digit year, milliseconds, and Z
}
//read the iso form back to its tick, or throw because the text isn't exactly what a tick writes
export function isoToTick(s) {
  let m = isoPattern.exec(s)
  let t = -1 //not a tick, until the text proves otherwise
  if (m) {
    let [y, mo, d, h, i, sec, ms] = m.slice(1).map(Number)
    t = Date.UTC(y, mo - 1, d, h, i, sec, ms) //a month is 0 through 11 here, and february 30 rolls into march
  }
  if (!(t >= 0 && t <= lastTick) || tickToIso(t) != s)
    throw new Error(`not the iso form of a tick: ${JSON.stringify(s)}`) //the round trip is the final check, catching the rolled february 30 and a year under 100 that Date.UTC moved to the 1900s
  return t
}

//write a tick in the short form, like "2012aug6.0517.57300", as deep as the moment needs and no deeper
export function tickToText(t) {
  let d = new Date(_checkTick(t))
  let y = d.getUTCFullYear()
  let m = d.getUTCMonth() + 1 //1 january through 12 december
  let day = d.getUTCDate()
  let h = d.getUTCHours()
  let i = d.getUTCMinutes()
  let ms = d.getUTCSeconds() * 1000 + d.getUTCMilliseconds() //milliseconds into the minute, 0 through 59999

  let depth = 0 //how deep the text must go: the year alone, then the month, day, hour, minute, and millisecond
  if (m > 1) depth = 1
  if (day > 1) depth = 2
  if (h > 0) depth = 3
  if (i > 0) depth = 4
  if (ms > 0) depth = 5
  let text = String(y)
  if (depth >= 1) text += months[m - 1]
  if (depth >= 2) text += day
  if (depth >= 3) text += '.' + String(h).padStart(2, '0')
  if (depth >= 4) text += String(i).padStart(2, '0')
  if (depth >= 5) text += '.' + ms
  return text
}
//read the short form back to its tick, or throw because the text isn't exactly what a tick writes
export function textToTick(s) {
  let t = _parseText(s)
  if (t === false || tickToText(t) != s)
    throw new Error(`not the short form of a tick: ${JSON.stringify(s)}`) //the round trip is the final check, catching february 30 and a padded or deeper text alike
  return t
}
//read the short form leniently enough to compute a tick, or false; textToTick above makes it strict with the round trip
function _parseText(s) {
  if (typeof s != 'string') return false
  let parts = s.split('.') //the year, month, and day; then the clock; then the milliseconds into the minute
  if (parts.length > 3) return false
  let p1 = parts[0]
  if (![4, 7, 8, 9].includes(p1.length) || !_isNumerals(p1.slice(0, 4)))
    return false //"1990", "1990feb", "1990feb2", "1990feb28"
  let y = Number(p1.slice(0, 4))
  let m = 1
  let day = 1
  let h = 0
  let i = 0
  let ms = 0
  if (p1.length >= 7) {
    m = months.indexOf(p1.slice(4, 7)) + 1 //0 if the month name isn't one of ours
    if (m == 0) return false
  }
  if (p1.length >= 8) {
    if (!_isNumerals(p1.slice(7))) return false
    day = Number(p1.slice(7))
  }
  if (parts.length >= 2) {
    let p2 = parts[1]
    if (!_isNumerals(p2) || (p2.length != 2 && p2.length != 4)) return false //"09" an hour, "0905" an hour and minute
    h = Number(p2.slice(0, 2))
    if (p2.length == 4) i = Number(p2.slice(2))
  }
  if (parts.length >= 3) {
    if (!_isNumerals(parts[2])) return false
    ms = Number(parts[2])
  }
  if (y < 1970 || day < 1 || day > 31 || h > 23 || i > 59 || ms > 59999)
    return false //bounds on each unit; the round trip rules out february 30
  return Date.UTC(y, m - 1, day, h, i) + ms
}

//  _
// | |_ __ _  __ _
// | __/ _` |/ _` |
// | || (_| | (_| |
//  \__\__,_|\__, |
//           |___/

export const tagLength = 21 //long enough to be unique, short enough to be reasonable, and nanoid's default

//letters and numerals only, so a double-click selects the whole tag; nanoid's default alphabet also has - and _
const tagAlphabet =
  '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'

//generate a new universally unique tag of 21 letters and numerals, like "AgKxru95C7jFp5iPuK9O7"
export function Tag() {
  //the algorithm is nanoid's: mask random bytes down to the alphabet's size, and skip the few values that land past its end, so every character is equally likely
  let mask = 63 //the alphabet has 62 characters; 63 is the smallest all-ones mask that covers them
  let step = Math.ceil((1.6 * mask * tagLength) / tagAlphabet.length) //random bytes per draw; 1.6 is nanoid's margin so one draw almost always suffices
  let tag = ''
  while (true) {
    let bytes = crypto.getRandomValues(new Uint8Array(step))
    for (let b of bytes) {
      let i = b & mask //0 through 63
      if (i < tagAlphabet.length) tag += tagAlphabet[i] //62 and 63 fall past the end; skipping them keeps the distribution uniform
      if (tag.length == tagLength) return tag
    }
  }
}

//true if s is exactly 21 letters and numerals
export function hasTag(s) {
  return (
    typeof s == 'string' && s.length == tagLength && /^[0-9A-Za-z]+$/.test(s)
  )
}

//a guard: pass a tag through, or throw because the caller handed us something that isn't one
export function checkTag(s) {
  if (!hasTag(s)) throw new Error(`not a tag: ${JSON.stringify(s)}`)
  return s
}

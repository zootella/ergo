//ergo, short for ergonomic: simple helper functions, the same in javascript, python, rust, and go

//noble's hashes, plain javascript that runs the same in a page, a web worker, and node, for what the platform's web crypto can't do; the hash section below says what that is
import { sha1 as nobleSha1 } from '@noble/hashes/legacy.js'
import { sha256 as nobleSha256 } from '@noble/hashes/sha2.js'
import { blake3 as nobleBlake3 } from '@noble/hashes/blake3.js'

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

Every function that takes a tick throws on a number that isn't one, because a caller handing it one has a bug. A number from outside, like a system clock or a file's modified time, isn't the caller's to vouch for, so hasTick asks about it first, and asking never fails; once hasTick says yes, nothing here throws on that tick. checkTick is the guard that enforces the same answer, and it's the gate every function here goes through.
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

//true if t is a tick, an integer from zero through the last millisecond of 9999; never throws, so a number from outside can be asked about first
export function hasTick(t) {
  return Number.isSafeInteger(t) && t >= 0 && t <= lastTick
}
//a guard: pass a tick through, or throw because the caller handed us something that isn't one
export function checkTick(t) {
  if (!hasTick(t)) throw new Error(`not a tick: ${t}`)
  return t
}
//true if s is made of numerals 0 through 9 and nothing else
function _isNumerals(s) {
  return /^[0-9]+$/.test(s)
}

//write a tick in the iso form, like "2012-08-06T05:17:57.300Z"
export function tickToIso(t) {
  return new Date(checkTick(t)).toISOString() //the platform writes exactly the form we read: a four digit year, milliseconds, and Z
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
  let d = new Date(checkTick(t))
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

//  _               _
// | |__   __ _ ___| |__
// | '_ \ / _` / __| '_ \
// | | | | (_| \__ \ | | |
// |_| |_|\__,_|___/_| |_|
//

/*
Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.

Each hash comes in two forms that give the same answer. The function takes all the bytes at once, like await sha256(data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with Sha256(), call update with each piece in order, and call digest for the hash. A file of any size hashes this way in the memory of one piece, and a single read of the file can feed all three hashers. Digest doesn't finish a hasher: update it again and digest again for the hash of everything so far.

The platform's Web Crypto has SHA-1 and SHA-256 all at once, at full native speed, so sha1 and sha256 are Web Crypto, and answer with a promise, as it does; blake3 answers with a promise too, so the three read alike. Web Crypto has no hasher that takes a piece at a time, and no BLAKE3 at all, so the three hashers, and blake3 all at once, come from noble's hashes: plain JavaScript with no dependencies of its own, the same in a page, a web worker, and Node. The hashers are synchronous, so a page with a large file should hash in a web worker, or between the reads of a stream, which give the page a moment between pieces. And in a browser, Web Crypto is there only in a secure context, a page from https or localhost and its workers, so on a plain http page sha1 and sha256 refuse, where the hashers still work; node always has it. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.
*/

//the SHA-1 hash of data, a Uint8Array, as a promise of 20 bytes; for matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions
export async function sha1(data) {
  return new Uint8Array(await crypto.subtle.digest('SHA-1', data)) //web crypto answers with an ArrayBuffer; ours is a Uint8Array, like everything else here
}
//the SHA-256 hash of data, a Uint8Array, as a promise of 32 bytes
export async function sha256(data) {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', data))
}
//the BLAKE3 hash of data, a Uint8Array, as a promise of 32 bytes; noble's is synchronous, and the promise is so all three read alike
export async function blake3(data) {
  return nobleBlake3(data)
}

//a SHA-1 hasher: update(piece) with each piece in order, then digest() for the same 20 bytes sha1 gives all at once; digest doesn't finish it, so update again and digest again for the hash of everything so far
export function Sha1() {
  return _hasher(nobleSha1.create())
}
//a SHA-256 hasher, the same way, for 32 bytes
export function Sha256() {
  return _hasher(nobleSha256.create())
}
//a BLAKE3 hasher, the same way, for 32 bytes
export function Blake3() {
  return _hasher(nobleBlake3.create())
}
//wrap one of noble's hashers in ours, which has just the two methods every edition shares
function _hasher(h) {
  let o = {}
  o.update = (data) => {
    h.update(data) //noble's update returns itself for chaining; ours returns nothing, like the other three editions'
  }
  o.digest = () => h.clone().digest() //from a copy, so the hasher keeps going, where noble's own digest would end it
  return o
}

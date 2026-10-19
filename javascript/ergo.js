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

//      _       _
//   __| | __ _| |_ __ _
//  / _` |/ _` | __/ _` |
// | (_| | (_| | || (_| |
//  \__,_|\__,_|\__\__,_|
//

/*
A Data holds bytes. Every language keeps bytes its own way, in arrays and buffers and views with their own rules about copying and size, and every platform call that takes or gives bytes wants its own one of them. A Data makes those choices once: it holds the bytes, and it speaks every form they travel in, text and base16, base32, base62, and base64, through methods with plain names. Make one from any form, read it out as any other, and it's the same bytes. So a Data is the point between all the other ways of holding bytes, even in JavaScript alone, where it trades between Uint8Array, ArrayBuffer, and text on the way in and out of the platform's own calls.

Every function in ergo that trades in bytes trades in Data. A hash takes a Data and answers one, and never decides what text the caller wants next; the caller says, with the next method. Only Data knows the encodings.

Each form is exact, both ways. Every reader ends by writing the bytes back out and requiring the text it read, so it accepts only what its writer produces: base16 in lowercase, base32 without padding, base64 with it, nothing extra and nothing missing. Text becomes UTF-8 in Unicode Normalization Form C, so the same characters typed two ways become the same bytes, and reading bytes as text refuses bytes that aren't UTF-8, or aren't in that form, rather than guess.

clipView looks at part of a Data without copying it, which is fast, and right whenever the bytes beneath won't change. clipCopy copies them out, for when they could. The same holds for array, which hands over the bytes themselves.
*/

//a Data wraps Uint8Array for type and bounds checks and format conversion
//make one from exactly one of {array}, {buffer}, {text}, {base16}, {base32}, {base62}, {base64}, or {random: n} for n bytes from the operating system's random source
export function Data(p) {
  let _array, _text, _base16, _base32, _base62, _base64 //private members hang out in this function the returned object came from, and are still here later!

  //constructor: a Data always contains an array, keeps the given form, and makes and keeps the others as requested
  if (p.array instanceof Uint8Array) {
    _array = p.array //just save the given array, without copying it
  } else if (p.buffer instanceof ArrayBuffer) {
    _array = new Uint8Array(p.buffer) //put a uint8 array view over the buffer
  } else if (typeof p.text == 'string') {
    _text = _normalText(p.text) //keep the text in the form its bytes hold, so reading it back gives the same text
    _array = _textToArray(_text)
  } else if (typeof p.base16 == 'string') {
    _array = _base16ToArray(p.base16)
    _base16 = p.base16
  } else if (typeof p.base32 == 'string') {
    _array = _base32ToArray(p.base32)
    _base32 = p.base32
  } else if (typeof p.base62 == 'string') {
    _array = _base62ToArray(p.base62)
    _base62 = p.base62
  } else if (typeof p.base64 == 'string') {
    _array = _base64ToArray(p.base64)
    _base64 = p.base64
  } else if (p.random != undefined) {
    _array = _randomArray(p.random)
  } else {
    throw new Error(
      `Data takes one of array, buffer, text, base16, base32, base62, base64, or random: ${JSON.stringify(p)}`,
    )
  }

  let d = { type: 'Data' } //note the type
  d.size = () => _array.length //size in bytes
  d.array = () => _array //the bytes themselves, which this doesn't copy
  d.text = () => {
    if (_text == undefined) _text = _arrayToText(_array)
    return _text
  }
  d.base16 = () => {
    if (_base16 == undefined) _base16 = _arrayToBase16(_array)
    return _base16
  }
  d.base32 = () => {
    if (_base32 == undefined) _base32 = _arrayToBase32(_array)
    return _base32
  }
  d.base62 = () => {
    if (_base62 == undefined) _base62 = _arrayToBase62(_array)
    return _base62
  }
  d.base64 = () => {
    if (_base64 == undefined) _base64 = _arrayToBase64(_array)
    return _base64
  }

  //get the byte at index i, a number 0x00 0 through 0xff 255
  d.get = (i) => {
    if (!(Number.isSafeInteger(i) && i >= 0 && i < _array.length))
      throw new Error(`no byte ${i} in ${_array.length} bytes`)
    return _array[i]
  }

  //Data's .array() and .clipView() do not copy bytes; use them to be fast when you know the buffer beneath won't change
  d.clipView = (start, end) => {
    _checkClip(_array.length, start, end)
    return Data({ array: _array.subarray(start, end) })
  }
  //alternatively, use .clipCopy() when you know the buffer beneath could change, and want the data as it is right now 🪏
  d.clipCopy = (start, end) => {
    _checkClip(_array.length, start, end)
    return Data({ array: _array.slice(start, end) })
  }
  return d
}

//a guard: pass a Data through, or throw because the caller handed us something else
function _checkData(d) {
  if (d?.type != 'Data') throw new Error(`expected a Data: ${d}`)
  return d
}
//start and end are byte indices from the start, with start through end the clip, so a clip of nothing is fine, but not one past the end, or backwards
function _checkClip(size, start, end) {
  if (!(
    Number.isSafeInteger(start) &&
    Number.isSafeInteger(end) &&
    start >= 0 &&
    start <= end &&
    end <= size
  ))
    throw new Error(`not a clip of ${size} bytes: ${start} to ${end}`)
}
//n is a count, zero or more, within the integer ceiling
function _checkCount(n) {
  if (!(Number.isSafeInteger(n) && n >= 0)) throw new Error(`not a count: ${n}`)
  return n
}

//n random bytes; getRandomValues fills at most 65536 at a call, so fill in steps
function _randomArray(n) {
  let a = new Uint8Array(_checkCount(n))
  for (let i = 0; i < a.length; i += 65536)
    crypto.getRandomValues(a.subarray(i, Math.min(i + 65536, a.length)))
  return a
}

//the last check in every reader: write the bytes back out, and require exactly the text it read
function _roundTrip(form, s, a, write) {
  if (write(a) != s) throw new Error(`not ${form}: ${JSON.stringify(s)}`)
  return a
}

let _textEncoder, _textDecoder //make once and use many times, saves no state between uses
//text in unicode Normalization Form C, so surrogate pairs and accents hash and match consistently however they were typed
function _normalText(s) {
  if (!s.isWellFormed()) throw new Error(`not text: ${JSON.stringify(s)}`) //a lone surrogate, half of a pair, has no UTF-8 form
  return s.normalize('NFC')
}
function _textToArray(s) {
  if (!_textEncoder) _textEncoder = new TextEncoder()
  return _textEncoder.encode(s) //returns a Uint8Array
}
function _arrayToText(a) {
  if (!_textDecoder)
    _textDecoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }) //fatal to throw rather than paper over bytes that aren't UTF-8, and ignoreBOM to keep a leading byte order mark as the character it is, rather than quietly drop it
  let s = _textDecoder.decode(a)
  if (s.normalize('NFC') != s)
    throw new Error(`not text in Normalization Form C: ${_arrayToBase16(a)}`) //text from bytes that are UTF-8 but not NFC would become different bytes again
  return s
}

function _arrayToBase16(a) {
  return Array.from(a, (byte) => byte.toString(16).padStart(2, '0')).join('')
}
function _base16ToArray(s) {
  if (s.length % 2 != 0) throw new Error(`not base16: ${JSON.stringify(s)}`)
  let a = new Uint8Array(s.length / 2)
  for (let i = 0; i < a.length; i++)
    a[i] = parseInt(s.slice(i * 2, i * 2 + 2), 16)
  return _roundTrip('base16', s, a, _arrayToBase16) //which refuses uppercase, and any pair parseInt read leniently or couldn't read at all
}

//b is a string where each character has the char code of a byte, 0-255; it doesn't print well
function _arrayToBase64(a) {
  let b = ''
  for (let i = 0; i < a.length; i += 8192)
    b += String.fromCharCode(...a.subarray(i, i + 8192)) //in steps, since spreading a large array into one call would overflow the stack
  return btoa(b)
}
function _base64ToArray(s) {
  let b
  try {
    b = atob(s)
  } catch {
    throw new Error(`not base64: ${JSON.stringify(s)}`) //the bottom gate: atob speaks an exception for text it can't read, and we answer with our own
  }
  let a = new Uint8Array(b.length)
  for (let i = 0; i < a.length; i++) a[i] = b.charCodeAt(i)
  return _roundTrip('base64', s, a, _arrayToBase64) //which refuses missing padding and whitespace, both of which atob forgives
}

//  _                    _________
// | |__   __ _ ___  ___|___ /___ \
// | '_ \ / _` / __|/ _ \ |_ \ __) |
// | |_) | (_| \__ \  __/___) / __/
// |_.__/ \__,_|___/\___|____/_____|
//

//base32, for hash values in the database, so they can be short, but also always the same length
const base32Alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'
let _base32Decoder //lookup table for decoding, made once on first use
function _arrayToBase32(a) {
  let s = '' //encoded string output
  let bits = 0 //bit counter
  let value = 0 //holds the accumulated bits
  for (let i = 0; i < a.length; i++) {
    value = (value << 8) | a[i] //shift value left by 8 bits and OR with the current byte
    bits += 8
    while (bits >= 5) {
      //while there are enough bits for a base32 character
      s += base32Alphabet[(value >>> (bits - 5)) & 31] //extract the top 5 bits as a base32 character
      bits -= 5
    }
  }
  if (bits > 0) s += base32Alphabet[(value << (5 - bits)) & 31] //handle the remaining bits
  return s
}
function _base32ToArray(s) {
  if (!_base32Decoder) {
    //on first use, make the lookup array
    _base32Decoder = new Uint8Array(256)
    for (let i = 0; i < base32Alphabet.length; i++)
      _base32Decoder[base32Alphabet.charCodeAt(i)] = i //map base32 characters to their indices
  }
  let bits = 0 //bit counter
  let value = 0 //holds the accumulated bits
  let a = [] //decoded bytes
  for (let i = 0; i < s.length; i++) {
    value = (value << 5) | _base32Decoder[s.charCodeAt(i)] //a character that isn't in the alphabet reads as 0 here, and the round trip refuses it
    bits += 5
    if (bits >= 8) {
      //when there are enough bits for a byte
      a.push((value >>> (bits - 8)) & 255) //extract the top 8 bits as a byte
      bits -= 8
    }
  }
  return _roundTrip('base32', s, new Uint8Array(a), _arrayToBase32) //which refuses padding, lowercase, and a last character with bits that don't belong to any byte
}

//  _                     __  ____
// | |__   __ _ ___  ___ / /_|___ \
// | '_ \ / _` / __|/ _ \ '_ \ __) |
// | |_) | (_| \__ \  __/ (_) / __/
// |_.__/ \__,_|___/\___|\___/_____|
//

/*
Why so many different encoding formats? base16 is good because it's simple, standard, and you can see the bytes. base32 is shorter, double-clickable, and always the same length, good for storing hash values in the database. base62 is shorter and double-clickable, good for bytes in the query string of api calls and the location bar the user might see. base64 is about the same length, standard, but not double-clickable, included here for completeness.
*/

const alphabet62Stream =
  '1023456789ABCDEFIGHJKLMNOPQRSTUVjWXYZabcdefghkmnlopqrstuvwxyzi' //narrow characters in popular spots, for data
/* ^               ^               ^               ^            ^
   1               I               j               l            i  moved characters
   0               16              32              48           61  to these indices

Encoding random short data, some characters appear more frequently than others.
The last character, at index 61, is the most common.
Also, the characters at 0, 16, 32, and 48 are slightly more common.
Rendered to pixels for a user, we want base62 text to be as short as possible!
So, we've adjusted the alphabet to put narrow characters in the popular spots.
*/

//turn data into text using base 62, each 4 or 6 bits will become a character 0-9, a-z, and A-Z
function _arrayToBase62(a) {
  //loop through the memory, encoding its bits into letters and numbers
  let i = 0 //the index in bits, from 0 through all the bits in the given data
  let byteIndex, bitIndex //the same index as a distance in bytes followed by a distance in bits
  let pair, mask, code //look at a pair of bytes at a time, with a mask of six 1s, to read a code 0 through 63
  let s = '' //target string to build up and return
  while (i < a.length * 8) {
    //when the bit index moves beyond the memory, we're done

    //calculate our byte and bit position for this run of the loop
    byteIndex = Math.floor(i / 8) //divide by 8 and chop off the remainder to get the byte index
    bitIndex = i % 8 //the bit index within that byte is the remainder

    //copy the two bytes at byteIndex into pair
    pair = (a[byteIndex] & 0xff) << 8 //copy the byte at byteIndex into pair, shifted left to bring eight 0s on the right
    if (byteIndex + 1 < a.length) pair |= a[byteIndex + 1] & 0xff //on the very last byte, leave the right byte in pair all 0s

    //read the 6 bits at i as a number, called code, which will be 0 through 63
    mask = 63 << (10 - bitIndex) //start the mask 111111 63 shifted into position     0011111100000000
    code = pair & mask //use the mask to clip out just that portion of pair         --101101--------
    code = code >>> (10 - bitIndex) //shift it to the right to read it as a number       ----------101101

    //describe the 6 bits with a numeral or letter, 111100 is 60, if 61 move forward 4, not 6
    if (code < 61) {
      s += alphabet62Stream.charAt(code) //000000 0 through 111100 60 append character at that index
      i += 6
    } else {
      s += alphabet62Stream.charAt(61) //111101 61, 111110 62, and 111111 63 append 'i' and move past '1111'
      i += 4
    }
  }
  return s
}
function _base62ToArray(s) {
  let b = Bin(s.length) //more characters become fewer bytes, so this capacity will always be plenty

  //loop for each character in the text
  let c //the character we are converting into bits
  let code //the bits the character gets turned into
  let hold = 0 //a place to hold bits from several characters until we have 8 and can write a byte
  let bits = 0 //the number of bits stored in the right side of hold right now
  for (let i = 0; i < s.length; i++) {
    //get a character from the text
    c = s.charAt(i)
    code = alphabet62Stream.indexOf(c)
    if (code < 0) throw new Error(`not base62: ${JSON.stringify(s)}`)

    //insert the bits from code into hold
    if (code == 61) {
      hold = (hold << 4) | 15 //insert 1111 for common 'i' at last alphabet index 61
      bits += 4
    } else {
      hold = (hold << 6) | code //insert 000000 through 111100 for other characters
      bits += 6
    }

    //if we have enough bits in hold to write a byte
    if (bits >= 8) {
      b.add((hold >>> (bits - 8)) & 0xff) //move the 8 leftmost bits in hold to our Bin object
      bits -= 8 //remove the bits we wrote from hold, any extra bits there will be written next time
    }
  }
  return _roundTrip('base62', s, b.array(), _arrayToBase62) //which refuses a last character with bits that don't belong to any byte
}

//  _     _
// | |__ (_)_ __
// | '_ \| | '_ \
// | |_) | | | | |
// |_.__/|_|_| |_|
//

//a Bin holds a set number of bytes, which you add a byte or a Data at a time until it's full
//it's how to assemble bytes whose size you know before you start, and refusing to hold more than that catches a mistake in the prediction
export function Bin(capacity) {
  _checkCount(capacity)

  //private members hang out in this function the return object came from, and are still here later!
  let _capacity = capacity //how many bytes it can hold
  let _size = 0 //how many bytes it does hold
  let _buffer = new ArrayBuffer(_capacity)
  let _array = new Uint8Array(_buffer) //view on the buffer that does unsigned 8 bit numbers like 0x00 through 0xff

  let b = { type: 'Bin' } //note the type
  b.capacity = () => _capacity //how many bytes it can hold
  b.size = () => _size //how many bytes it does hold
  b.add = (p) => {
    if (typeof p == 'number') {
      if (!(Number.isInteger(p) && p >= 0 && p <= 255))
        throw new Error(`not a byte: ${p}`)
      if (_size + 1 > _capacity)
        throw new Error(`a Bin of ${_capacity} bytes is full`)
      _array[_size] = p
      _size++
    } else if (p?.type == 'Data') {
      if (_size + p.size() > _capacity)
        throw new Error(
          `a Bin of ${_capacity} bytes holding ${_size} can't take ${p.size()} more`,
        )
      _array.set(p.array(), _size)
      _size += p.size()
    } else {
      throw new Error(`a Bin takes a byte or a Data: ${p}`)
    }
  }

  //Bin's .array() and .data() do not copy bytes; use them to always see the current contents of _buffer
  b.array = () => new Uint8Array(_buffer, 0, _size)
  b.data = () => Data({ array: b.array() })
  return b
}

//  _               _
// | |__   __ _ ___| |__
// | '_ \ / _` / __| '_ \
// | | | | (_| \__ \ | | |
// |_| |_|\__,_|___/_| |_|
//

/*
Three hashes, each the one the world already knows. SHA-1 is the oldest, the one older manifests name a file by and git names its objects with. It's broken for collisions, two different inputs made to share a hash on purpose, so use it to match a hash that already exists, never to protect anything. SHA-256 is the standard everywhere today. BLAKE3 is the newest, built for speed as a tree, so pieces of one input can hash on many cores at once, and it's the one to name a file by for years to come.

Each hash takes a Data and answers one, and comes in two forms that give the same answer. The function takes all the bytes at once, like await sha256(data), for data already in memory. The hasher takes them a piece at a time, as a file or a stream delivers them: make one with Sha256(), call update with each piece in order, and call digest for the hash. A file of any size hashes this way in the memory of one piece, and a single read of the file can feed all three hashers. Digest doesn't finish a hasher: update it again and digest again for the hash of everything so far.

The platform's Web Crypto has SHA-1 and SHA-256 all at once, at full native speed, so sha1 and sha256 are Web Crypto, and answer with a promise, as it does; blake3 answers with a promise too, so the three read alike. Web Crypto has no hasher that takes a piece at a time, and no BLAKE3 at all, so the three hashers, and blake3 all at once, come from noble's hashes: plain JavaScript with no dependencies of its own, the same in a page, a web worker, and Node. The hashers are synchronous, so a page with a large file should hash in a web worker, or between the reads of a stream, which give the page a moment between pieces. And in a browser, Web Crypto is there only in a secure context, a page from https or localhost and its workers, so on a plain http page sha1 and sha256 refuse, where the hashers still work; node always has it. Each edition takes each hash from its platform wherever the platform has it, and otherwise from the official implementation or the best known one, and the shared vectors and the fuzz test hold all four to the same answers.
*/

//the SHA-1 hash of a Data, as a promise of a Data of 20 bytes; for matching a hash that already exists, like an older manifest's, since SHA-1 is broken for collisions
export async function sha1(data) {
  return Data({
    buffer: await crypto.subtle.digest('SHA-1', _checkData(data).array()),
  }) //web crypto answers with an ArrayBuffer, which Data takes as it is
}
//the SHA-256 hash of a Data, as a promise of a Data of 32 bytes
export async function sha256(data) {
  return Data({
    buffer: await crypto.subtle.digest('SHA-256', _checkData(data).array()),
  })
}
//the BLAKE3 hash of a Data, as a promise of a Data of 32 bytes; noble's is synchronous, and the promise is so all three read alike
export async function blake3(data) {
  return Data({ array: nobleBlake3(_checkData(data).array()) })
}

//the SHA-256 hash of text, as base32, the shortcut for the common case; text to Data, hashed, and out as the form icarus keeps hash values in
export async function hashText(s) {
  return (await sha256(Data({ text: s }))).base32()
}

//a SHA-1 hasher: update(piece) with each piece in order, each a Data, then digest() for the same Data of 20 bytes sha1 gives all at once; digest doesn't finish it, so update again and digest again for the hash of everything so far
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
    h.update(_checkData(data).array()) //noble's update returns itself for chaining; ours returns nothing, like the other three editions'
  }
  o.digest = () => Data({ array: h.clone().digest() }) //from a copy, so the hasher keeps going, where noble's own digest would end it
  return o
}

//  _               _
// | |__   __ _ ___| |__    _ __ ___   ___  __ _ ___ _   _ _ __ ___
// | '_ \ / _` / __| '_ \  | '_ ` _ \ / _ \/ _` / __| | | | '__/ _ \
// | | | | (_| \__ \ | | | | | | | | |  __/ (_| \__ \ |_| | | |  __/
// |_| |_|\__,_|___/_| |_| |_| |_| |_|\___|\__,_|___/\__,_|_|  \___|
//

/*
The fuji hashes are two of ergo's own, from icarus, and both are SHA-256 over a summary of a file rather than over the file itself. The piece hash covers every byte: it cuts the file into 4 MiB pieces, hashes each one, and hashes the list of those hashes. The tip hash covers almost none of them: it hashes the file's first 4 KiB, 4 KiB from its middle, and its last two, at most 16 KiB however large the file, so it's done in a few reads and finds likely copies of a file across millions of others for the cost of those few reads each. A file of 16 KiB or less is its own tip. Each summary starts with a title that names the protocol and the file's size, like "Fuji.Tips.SHA256.4KiB.5.", so two files of different sizes never share a hash, even with identical tips.

The pieces are there by design. Web Crypto hashes only all at once, so a page can't hash a whole large file through it without holding all of it in memory; hashing 4 MiB pieces and then the list of them lets a page hash a file of any size at Web Crypto's full native speed, a piece at a time. And the tip hash can come two ways that must agree: from a File, by slicing out just the stripes, or from a stream, by noticing the stripes as they pass on the way to the piece hash.
*/

//given a file size and a unit, measure the pieces of that size, and the stripes the tip hash reads 📏
export function hashMeasure({ file, unit }) {
  if (!(
    Number.isSafeInteger(file) &&
    Number.isSafeInteger(unit) &&
    file >= 1 &&
    unit >= 1
  ))
    throw new Error(`not a file and unit to measure: ${file}, ${unit}`) //both the file size and piece size must be 1 or more bytes
  let o = { file, unit }

  o.pieces = Math.ceil(file / unit) //total number of pieces
  o.complete = Math.floor(file / unit) //quantity of complete pieces
  o.fragment = file % unit //byte size of last piece if it is a fragment

  o.stripes = []
  if (file > 4 * unit) {
    //place indices
    o.first = 0
    o.middle = Math.floor(file / (2 * unit)) * unit //byte index of middle piece
    o.penultimate = (o.pieces - 2) * unit //second to last piece, must always be complete
    o.last = (o.pieces - 1) * unit //last piece, can be complete, likely a fragment

    //calculate sizes
    o.lastSize = o.fragment > 0 ? o.fragment : unit //how big the last stripe is
    o.stripeSize = 3 * unit + o.lastSize //how many bytes we'll hash through all four stripes

    //build stripes; they are all [start, end] both measured from the start of the file, not start and length
    o.stripes.push([o.first, o.first + unit]) //first and middle never touch
    if (o.middle + unit == o.penultimate) {
      //but middle and penultimate can
      o.stripes.push([o.middle, o.middle + unit + unit + o.lastSize]) //and penultimate and last always do
    } else {
      o.stripes.push([o.middle, o.middle + unit]) //middle with space before and after
      o.stripes.push([o.penultimate, o.penultimate + unit + o.lastSize])
    }
  } else {
    o.all = true //file size is 4 units or less, so just hash the whole thing
    o.stripeSize = file
    o.stripes.push([0, file])
  }
  return o
}

//given a foreground stripe [a, b] and a background pattern of stripes [[c, d], ...], determine and calculate the overlapping ranges
//the background stripes must not touch or overlap, and indices are from the start, this is [start, end] not start and length
export function hashStripeOverlap(foreground, background) {
  let overlaps = []
  let [a, b] = foreground
  for (let [c, d] of background) {
    if (a < d && b > c) overlaps.push([Math.max(a, c), Math.min(b, d)]) //there's an overlap!
  }
  return overlaps //none at all is an empty list, where icarus answered null; one type loops the same either way
}

//  _               _        __ _ _                        _       _
// | |__   __ _ ___| |__    / _(_) | ___    __ _ _ __   __| |  ___| |_ _ __ ___  __ _ _ __ ___
// | '_ \ / _` / __| '_ \  | |_| | |/ _ \  / _` | '_ \ / _` | / __| __| '__/ _ \/ _` | '_ ` _ \
// | | | | (_| \__ \ | | | |  _| | |  __/ | (_| | | | | (_| | \__ \ |_| | |  __/ (_| | | | | | |
// |_| |_|\__,_|___/_| |_| |_| |_|_|\___|  \__,_|_| |_|\__,_| |___/\__|_|  \___|\__,_|_| |_| |_|
//

//a new simple protocol that can hash huge files on both the server and the page; introducing the "Fuji" system 🗻
export const hashProtocol = /*#__PURE__*/ Object.freeze({
  pieces: /*#__PURE__*/ Object.freeze({
    title: 'Fuji.Pieces.SHA256.4MiB.',
    size: 4194304, //4 * Size.mb, written out so a bundler can drop it; the vectors pin it
  }),
  tips: /*#__PURE__*/ Object.freeze({
    title: 'Fuji.Tips.SHA256.4KiB.',
    size: 4096, //4 * Size.kb
  }),
})
/*
4 MiB hashes in ~10ms similar to the frequency of page animation frames,
uploads in under a second over a typical cable modem, and
keeps the index of hash values smaller than the piece size for files under 500 GB 🗄️
4 KiB matches the cluster and sector size on Windows NTFS and the block size on Mac APFS,
so hard drive reads can be as fast as possible 💽
*/

//the Data a Bin holds, or throw because it isn't exactly full, which means we predicted wrong how much data we'd hash
function _full(bin) {
  if (bin.size() != bin.capacity())
    throw new Error(
      `a Bin of ${bin.capacity()} bytes holding ${bin.size()} isn't full`,
    )
  return bin.data()
}

//given a standard JavaScript File object (extends Blob) and size, quickly compute just the tip hash
//works in local Node testing and a browser page with Uppy, but not in Lambda Node! (where we can only get a stream)
export async function hashFile({ file, size, protocol = hashProtocol }) {
  if (!(file && size > 0 && file.size == size))
    throw new Error(`not a file of ${size} bytes: ${file}`) //file is a JavaScript File object, which extends Blob

  //based on the file size, pick stripes at the start, middle, and end for us to hash quickly
  let measureTips = hashMeasure({ file: size, unit: protocol.tips.size })
  let status = {
    startTime: Date.now(),
    updateTime: Date.now(), //when we last changed anything here
    measureTips: measureTips, //include this, too
    totalSize: size,
    hashedSize: 0, //haven't hashed anything yet; we don't have a progress callback that might look, but still
  }

  //for tip hashing, the summary we'll hash is the title followed by stripes of file data (hashing file data, not hashes)
  let tipsTitle = Data({ text: `${protocol.tips.title}${size}.` }) //different sized files hash differently even with identical tips
  let tipsBin = Bin(tipsTitle.size() + measureTips.stripeSize)
  tipsBin.add(tipsTitle)
  for (let [start, end] of measureTips.stripes) {
    _checkClip(size, start, end)
    tipsBin.add(Data({ buffer: await file.slice(start, end).arrayBuffer() }))
  }

  //hash the summary of the file in the bin, which must be exactly full, a check on our prediction of how much data we'd hash
  status.tipHash = await sha256(_full(tipsBin))
  status.hashedSize = measureTips.stripeSize //we actually hashed more because of the title, but don't count this as to speed
  status.updateTime = Date.now()
  status.duration = status.updateTime - status.startTime
  return status
}

//a fuji hasher, a piece at a time: make one for a file of a given size, then await update(piece) with each piece of the file in order, each a Data, and await digest() for both its tip hash and its piece hash
//it's for bytes that arrive as events, like the chunks of an upload; hashStream below reads a stream into one
//update answers a promise, since the pieces go through web crypto; awaiting each keeps a fast producer from piling up pieces not yet hashed, and each update waits for the one before, so pieces hash in order even when a caller doesn't
//a piece is read when its turn comes, so leave its bytes as they are until its update answers
//update never fails; digest refuses, by throwing, when the pieces didn't add up to the size, since that's a fact about the data that came in rather than a bug; and digest doesn't finish it, as with Sha256
export function FujiHasher({ size, protocol = hashProtocol }) {
  if (!(Number.isSafeInteger(size) && size > 0))
    throw new Error(`not a size to hash: ${size}`) //an empty file has no fuji hash; both measures need a size of at least one byte

  let measurePieces = hashMeasure({ file: size, unit: protocol.pieces.size }) //measurements for the piece hash
  let measureTips = hashMeasure({ file: size, unit: protocol.tips.size }) //and for the tip hash, which we'll peek for as the pieces pass

  //for the pieces hash, the summary we'll hash is the title followed by hashes of pieces of file data (hashing hashes, not file data)
  let piecesTitle = Data({ text: `${protocol.pieces.title}${size}.` })
  let piecesBin = Bin(piecesTitle.size() + 32 * measurePieces.pieces) //space for title followed by hashes of every piece, each SHA-256 hash 32 bytes
  piecesBin.add(piecesTitle)

  //for the tips hash, the summary we'll hash is the title followed by stripes of file data (hashing file data, not hashes)
  let tipsTitle = Data({ text: `${protocol.tips.title}${size}.` })
  let tipsBin = Bin(tipsTitle.size() + measureTips.stripeSize) //space for title followed by the start, middle, and end stripes of data
  tipsBin.add(tipsTitle)

  //our conveyor belt for hashing bytes then sliding them forward 🏗️
  let belt = {}
  belt.capacity = Math.min(
    2 * protocol.pieces.size, //double-wide to hold one full piece and up to all of the next one
    size, //or sized to fit all of a smaller file
  )
  belt.array = new Uint8Array(belt.capacity) //this method allocates a buffer once and uses it for the whole file!
  belt.fill = 0 //the belt has .fill bytes of data in it, measured from the start; data bytes are 0 up to belt.fill

  let address = 0 //how far into the file each box that arrives is
  let queue = Promise.resolve() //the updates so far, each waiting for the one before

  //take the next box of bytes 📦
  async function feed(data) {
    let box = {}
    box.data = data
    box.shoveled = 0 //we move this index over the bytes in the box as we process them; remaining data starts at box.shoveled
    box.stripe = [address, address + box.data.size()] //where in the file this box of data is
    address += box.data.size() //outside the box, move address past this box to use to mark the start of the next one
    if (address > size) return //more bytes than the size, which digest will refuse; past the size, there's nothing to hash them into

    //first, for the tip hash, we don't have random access through the file, but we do through this box!
    for (let [start, end] of hashStripeOverlap(
      box.stripe,
      measureTips.stripes,
    )) {
      //the 1-3 stripes in the file we should add to assemble the file data for the tip hash
      tipsBin.add(box.data.clipView(start - box.stripe[0], end - box.stripe[0])) //subtract box address to clip in data
    }

    //second, for the piece hash, shovel bytes from the box into the belt until the box is empty
    while (box.shoveled < box.data.size()) {
      //loop to get through them

      //copy a shovelful of bytes from the box to our belt 🪏
      let shovel = Math.min(
        //how many can we shovel over right now?
        box.data.size() - box.shoveled, //not more that remain for us to take from this delivery
        belt.capacity - belt.fill, //nor more than we have space left in our conveyor belt buffer
      )
      belt.array.set(
        //into belt...
        box.data.clipView(box.shoveled, box.shoveled + shovel).array(), //...copy shovel quantity bytes from box at index...
        belt.fill, //...to belt at its fill position
      )
      belt.fill += shovel //there are more bytes of file data in belt now
      box.shoveled += shovel //and we've moved past them in the box

      //if the belt has enough data at the start to hash
      while (belt.fill >= protocol.pieces.size) {
        //hash the first half; if the file filled 8mb all at once this loop will run twice!
        piecesBin.add(
          await sha256(
            Data({ array: belt.array }).clipView(0, protocol.pieces.size),
          ),
        ) //4mib hashes in ~10ms, frequency like animation frames

        //slide the second half of the conveyor belt buffer to the start
        let beyond = belt.fill - protocol.pieces.size //how many bytes of data are on the belt beyond the first half we just hashed
        if (beyond > 0)
          belt.array.copyWithin(0, protocol.pieces.size, belt.fill) //this calls down to C's memmove, and is very fast
        belt.fill = beyond
      }
    }
  }

  //the hashes of all the bytes taken in, which must add up to the size
  async function finish() {
    if (address != size)
      throw new Error(
        `the pieces were ${address} bytes, not the ${size} they were said to be`,
      )

    //most files will end with a fragment piece smaller than a full piece; hash it into a copy of the summary, so the hasher is unchanged and digest can be asked again
    let summary = Bin(piecesBin.capacity())
    summary.add(piecesBin.data())
    if (belt.fill > 0)
      summary.add(
        await sha256(Data({ array: belt.array }).clipView(0, belt.fill)),
      )

    //hash the summaries of the file in the bins, which must be exactly full, a check on our prediction of how much data we'd hash
    return {
      tipHash: await sha256(_full(tipsBin)),
      pieceHash: await sha256(_full(summary)),
    }
  }

  let o = {}
  o.update = (data) => {
    _checkData(data)
    queue = queue.then(() => feed(data))
    return queue
  }
  o.digest = () => queue.then(finish)
  return o
}

//given a ReadableStream (Web Streams API), compute both the tip hash and the piece hash
//works everywhere: local and Lambda Node, and Uppy on a page; it reads the stream into a FujiHasher, which bytes that arrive as events can feed directly
export async function hashStream({
  stream,
  size,
  protocol = hashProtocol,
  onProgress,
  signal,
}) {
  signal?.throwIfAborted()
  if (!(stream && size > 0))
    throw new Error(`not a stream of ${size} bytes: ${stream}`)
  let hasher = FujiHasher({ size, protocol })
  let status = {
    //object we'll give to the progress callback, and also return
    startTime: Date.now(),
    updateTime: Date.now(), //when we last changed anything here
    measurePieces: hashMeasure({ file: size, unit: protocol.pieces.size }), //the measurements the hasher works from, for the piece hash
    measureTips: hashMeasure({ file: size, unit: protocol.tips.size }), //and for the tip hash
    hashedSize: 0, //how much of the stream is hashed, a piece at a time
    totalSize: size,
  }

  let fed = 0 //how many bytes we've given the hasher
  const reader = stream.getReader()
  try {
    while (fed <= size) {
      //loop while boxes of data arrive from the stream; past the size, stop reading, and digest will refuse
      signal?.throwIfAborted()
      let r = await reader.read() //wait for the stream to give us something
      if (r.value?.length) {
        //a new delivery of bytes from the stream has arrived! 📦 testing in local node, boxes come in 64kib at a time, the belt holds 128 of that size
        await hasher.update(Data({ array: r.value }))
        fed += r.value.length
        let hashed = fed - (fed % protocol.pieces.size) //each update hashes every piece it fills, so this much is hashed
        if (hashed > status.hashedSize && hashed <= size) {
          //another piece is hashed, about every 10ms, frequency like animation frames
          status.hashedSize = hashed
          status.updateTime = Date.now()
          onProgress?.(status)
        }
      }
      if (r.done) break
    }

    //hash the summary of the file
    signal?.throwIfAborted()
    let { tipHash, pieceHash } = await hasher.digest()
    status.hashedSize = size //the fragment piece, if any, is hashed too
    status.tipHash = tipHash
    status.pieceHash = pieceHash
    status.updateTime = Date.now()
    status.duration = status.updateTime - status.startTime
    onProgress?.(status)
    return status
  } finally {
    //try with no catch so an exception in here throws upwards into caller
    reader.releaseLock() //but before we leave, success or exception, we always release the file lock
  }
}

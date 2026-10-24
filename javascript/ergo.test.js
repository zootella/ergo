import { test, expect, describe, beforeAll, afterAll } from 'vitest'
import {
  hasInt,
  checkInt,
  hasString,
  checkString,
  hasText,
  checkText,
  trim,
  cut,
  cutLast,
  cut2,
  cutAfterLast,
  deindent,
  Size,
  saySize4,
  Time,
  sayDuration,
  sayTimeRemaining,
  sayTimeRemainingCoarse,
  sayTimeRace,
  tickToText,
  textToTick,
  tickToIso,
  isoToTick,
  hasTick,
  sayTick,
  monthNames,
  dayNames,
  Tag,
  hasTag,
  checkTag,
  tagLength,
  sha1,
  sha256,
  blake3,
  Sha1,
  Sha256,
  Blake3,
  hashText,
  Data,
  Bin,
  hashFile,
  hashStream,
  FujiHasher,
  hasOutlineName,
  checkOutlineName,
  Outline,
  textToOutline,
  dataToOutline,
  dataToQuoted,
  quotedToData,
  spanEncode,
  spanDecode,
} from './ergo.js'
import { call } from './driver.js'
import { startEditions, stopEditions, chosenEditions } from './editions.js'
import { readdirSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

//                 _
// __   _____  ___| |_ ___  _ __ ___
// \ \ / / _ \/ __| __/ _ \| '__/ __|
//  \ V /  __/ (__| || (_) | |  \__ \
//   \_/ \___|\___|\__\___/|_|  |___/
//

//the editions this run drives, all four unless ERGO_EDITIONS names some, like ERGO_EDITIONS=javascript pnpm test; the blocks below that test javascript alone, in-process, run only when javascript is one of them
const chosen = chosenEditions()
const javascriptChosen = chosen.includes('javascript')

//the shared vectors beside this edition: every case from every json file in tests/
const vectorsDirectory = join(
  dirname(fileURLToPath(import.meta.url)),
  '..',
  'tests',
)
const vectors = readdirSync(vectorsDirectory)
  .filter((f) => f.endsWith('.json'))
  .flatMap((f) => JSON.parse(readFileSync(join(vectorsDirectory, f), 'utf8')))

//a call that refuses, the way a throws vector or a scripted refusal expects; an unknown function refuses everything, so it can't pass a throws vector, or a scripted refusal either
async function expectRefused(promise, label) {
  let refusal = await promise.then(
    () => undefined,
    (e) => e,
  )
  expect(refusal, `${label ?? ''} answered instead of refusing`).toBeInstanceOf(
    Error,
  )
  expect(refusal.message, label).not.toMatch(/unknown function: /)
}

//name a case the way it reads: saySize4(10000) five digits of bytes, so the next unit up
function describeVector(c) {
  return `${c.function}(${c.input.map((a) => JSON.stringify(a)).join(', ')}) ${c.note ?? ''}`
}

describe.skipIf(!javascriptChosen)('vectors, javascript in-process', () => {
  for (let c of vectors) {
    test(describeVector(c), async () => {
      if (c.throws) await expectRefused(call(c.function, c.input))
      else expect(await call(c.function, c.input)).toEqual(c.output)
    })
  }
})

//      _
//  ___(_)_______
// / __| |_  / _ \
// \__ \ |/ /  __/
// |___/_/___\___|
//

describe.skipIf(!javascriptChosen)('int', () => {
  test('asks whether a number is an integer every edition holds exactly', () => {
    // #region int
    let carried = hasInt(9007199254740991) //true: 2^53 - 1, the largest integer every edition holds exactly
    let past = hasInt(2 ** 53) //false: past the ceiling, where a double can't tell one integer from the next
    let checked = checkInt(1344230277300) //passes an integer through, or throws because the caller handed it something that isn't one
    // #endregion int
    expect([carried, past, checked]).toEqual([true, false, 1344230277300])
  })
})

//  _            _
// | |_ _____  _| |_
// | __/ _ \ \/ / __|
// | ||  __/>  <| |_
//  \__\___/_/\_\\__|
//

describe.skipIf(!javascriptChosen)('text', () => {
  test('trims text, asks whether it holds anything, cuts it at a tag, and takes the indent off a multiline string', () => {
    // #region text
    let name = trim('  Alice  ') //"Alice": whitespace off both ends, by unicode's list in every edition
    let blank = hasText('   ') //false: nothing there but whitespace
    let c = cut('key=value', '=') //found, with "key" before the tag and "value" after it
    let note = deindent`
      line one
        line two
    ` //"line one\n  line two\n": the first line's indent comes off every line
    // #endregion text
    expect(name).toBe('Alice')
    expect(blank).toBe(false)
    expect(c).toEqual({ found: true, before: 'key', tag: '=', after: 'value' })
    expect(note).toBe('line one\n  line two\n')
  })
  test('asks whether a value is a string, and whether a string is text', () => {
    expect(hasText('a')).toBe(true)
    expect(hasText(' a ')).toBe(true) //something besides whitespace, wherever it sits
    expect(hasText()).toBe(false) //nothing
    expect(hasText('')).toBe(false) //empty string
    expect(hasText(' ')).toBe(false) //just spaces
    expect(hasText(7)).toBe(false) //non-string
    expect(hasText('\t\r\n')).toBe(false) //nonprinting characters

    expect(hasString()).toBe(false)
    expect(hasString(5)).toBe(false)
    expect(hasString('')).toBe(true) //hasString requires a string, but blank is ok
    expect(hasString(' ')).toBe(true)
    expect(hasString('a')).toBe(true)

    expect(checkText(' a ')).toBe(' a ') //each check passes its value through
    expect(checkString('')).toBe('')
    expect(() => checkText(' ')).toThrow() //hasText and checkText want a string that's not blank and not trimmable to blank
    expect(() => checkString(5)).toThrow()
  })
  test("trims by unicode's list of whitespace, which differs from javascript's own at exactly two characters", () => {
    expect(trim('  a b  ')).toBe('a b') //off both ends, and never from the middle
    expect(trim(' \u00a0a\u3000')).toBe('a') //a no-break space and an ideographic space are whitespace too
    expect(trim('')).toBe('')
    expect(trim(' \t\n ')).toBe('')
    expect(() => trim(5)).toThrow()

    expect(trim('a\u0085')).toBe('a') //unicode calls the next line control whitespace, and javascript's trim doesn't
    expect('a\u0085'.trim()).toBe('a\u0085')
    expect(trim('\ufeffa')).toBe('\ufeffa') //javascript's trim takes off the byte order mark, which unicode doesn't call whitespace
    expect('\ufeffa'.trim()).toBe('a')
    expect(hasText('\ufeff')).toBe(true) //so a byte order mark alone is text
    expect(hasText('\u0085')).toBe(false) //and a next line control alone isn't

    let differ = [] //every character of the basic plane, where all 25 of unicode's whitespace characters live, trimmed both ways
    for (let c = 0; c <= 0xffff; c++) {
      if (c >= 0xd800 && c <= 0xdfff) continue //half of a surrogate pair isn't a character
      let s = String.fromCharCode(c)
      if (trim(s) != s.trim()) differ.push(c)
    }
    expect(differ).toEqual([0x85, 0xfeff])
  })
  test('cuts text around a tag', () => {
    let s, c
    s = 'red<a>green<a>blue'
    c = cut(s, '<a>')
    expect(c).toEqual({
      found: true,
      before: 'red',
      tag: '<a>',
      after: 'green<a>blue',
    })
    c = cutLast(s, '<a>')
    expect(c).toEqual({
      found: true,
      before: 'red<a>green',
      tag: '<a>',
      after: 'blue',
    })
    expect(c.before + c.tag + c.after).toBe(s) //we include tag so you can assemble s from given parts

    c = cut(s, '<b>')
    expect(c).toEqual({ found: false, before: s, tag: '', after: '' })
    c = cutLast(s, '<b>')
    expect(c).toEqual({ found: false, before: s, tag: '', after: '' }) //all before so you can move forward in a loop until after is blank

    c = cut2(s, '<a>', '<a>')
    expect(c).toEqual({
      found: true,
      before: 'red',
      tag1: '<a>',
      middle: 'green',
      tag2: '<a>',
      after: 'blue',
    })
    expect(cut2('red<a>green', '<a>', '<b>')).toEqual({
      found: false,
      before: 'red<a>green',
      tag1: '',
      middle: '',
      tag2: '',
      after: '',
    }) //both must be found

    expect(cutAfterLast('Some_Prefix_ValueValue', '_')).toBe('ValueValue')
    expect(() => cutAfterLast('NoPrefix', '_')).toThrow() //make sure s is what we expect
  })
  test('cuts at any tag of at least one character, whitespace included, and refuses a blank one', () => {
    expect(cut('first second', ' ')).toEqual({
      found: true,
      before: 'first',
      tag: ' ',
      after: 'second',
    }) //a space can be the tag
    expect(cut('line one\nline two', '\n').after).toBe('line two') //and so can a newline
    expect(cut('', 'x')).toEqual({
      found: false,
      before: '',
      tag: '',
      after: '',
    }) //blank text to search is fine

    expect(() => cut('abc', '')).toThrow() //a blank tag would be found at the start of everything
    expect(() => cutLast('abc', '')).toThrow()
    expect(() => cut('abc', 5)).toThrow()
    expect(() => cut(5, 'a')).toThrow()

    let words = [] //a loop that cuts until nothing is left, which a blank tag would never let end
    let rest = 'a b c'
    while (rest) {
      let c = cut(rest, ' ')
      words.push(c.before)
      rest = c.after
    }
    expect(words).toEqual(['a', 'b', 'c'])
  })
  test('deindent lets a multiline template literal indent with the code around it', () => {
    let name1 = 'Alice'
    let name2 = 'Bob'
    expect(deindent`Hello ${name1} and ${name2}`).toBe('Hello Alice and Bob\n')

    //now you can indent a multiline string with the surrounding code
    let s1 = deindent`
      1. Introduction
      2. Summary of Purpose
        2a. Detail about Particular Purpose
      3. Action Plan
    `
    //rather than having to do this
    let s2 = deindent`1. Introduction
2. Summary of Purpose
  2a. Detail about Particular Purpose
3. Action Plan
`
    expect(s1).toBe(s2) //and they're the same!
    expect(s1).toBe(
      '1. Introduction\n2. Summary of Purpose\n  2a. Detail about Particular Purpose\n3. Action Plan\n',
    )
  })
  test('demonstration of deindent, with the shape of its output in view', () => {
    let s = deindent`
      A
        B
      C
    `
    expect('\n↓\n' + s + '↑').toBe('\n↓\nA\n  B\nC\n↑') //newlines so you can see output between vertical arrows
  })
  test('deindent counts a tab as two spaces, takes whitespace off the end of each line, and refuses to be called as a function', () => {
    expect(deindent`
\t\tA
\t\t\tB
\t\tC
`).toBe('A\n  B\nC\n') //a tab at the start of a line is two spaces of indent
    expect(deindent`a\r\nb\rc`).toBe('a\nb\nc\n') //every kind of line ending becomes \n
    expect(deindent`
      a \u00a0
        \u3000
      b
    `).toBe('a\n\nb\n') //whitespace by unicode's list leaves the end of each line, and a line of it alone becomes a blank line that stays, here in the middle
    expect(deindent`${5} and ${null}`).toBe('5 and null\n') //fields fill in the way a template literal fills them
    expect(() => deindent('a')).toThrow() //you probably called this like a function
  })
})

describe.skipIf(!javascriptChosen)('Size', () => {
  test('names a number of bytes, and says one', () => {
    // #region size
    let buffer = new Uint8Array(64 * Size.kb) //a buffer of 64 kibibytes, rather than a bare 65536
    let said = saySize4(123456789) //"117 MB": four digits or less, with a unit
    // #endregion size
    expect(buffer.length).toBe(65536)
    expect(said).toBe('117 MB')
  })
})

//  _   _
// | |_(_)_ __ ___   ___
// | __| | '_ ` _ \ / _ \
// | |_| | | | | | |  __/
//  \__|_|_| |_| |_|\___|
//

describe.skipIf(!javascriptChosen)('Time', () => {
  test('names a span of milliseconds', () => {
    // #region time
    let deadline = 1700000000000 + 2 * Time.hour //two hours after a tick, which counts milliseconds since 1970
    // #endregion time
    expect(deadline).toBe(1700007200000)
  })
  test('says a span for a person', () => {
    // #region sayduration
    let took = sayDuration(2 * Time.hour + 5 * Time.minute, '# hours # minutes') //"2 hours 5 minutes": the template is the sentence, with # where each number goes
    let ago = sayDuration(3 * Time.day, 'about # days ago') //"about 3 days ago"
    let left = sayTimeRemaining(9 * Time.minute + 59 * Time.second) //"9m 59s": one or two units, whichever matter at the size
    let race = sayTimeRace(5 * Time.minute + 15 * Time.second + 223) //"5'15\"223": to the millisecond, the way a race is timed
    // #endregion sayduration
    expect([took, ago, left, race]).toEqual([
      '2 hours 5 minutes',
      'about 3 days ago',
      '9m 59s',
      `5'15"223`,
    ])
  })
  test('demonstration of sayDuration, a guided tour of saying a span of time', () => {
    //a template is the sentence you want, with a # wherever a number goes and a unit after each #
    expect(
      sayDuration(2 * Time.hour + 5 * Time.minute, '# hours # minutes'),
    ).toBe('2 hours 5 minutes')

    //whole words come out singular for exactly 1, so write the plural and let sayDuration choose
    expect(sayDuration(Time.hour + Time.minute, '# hours # minutes')).toBe(
      '1 hour 1 minute',
    )

    //name the units you want, and the largest takes everything above it: a year in days, in weeks, or in hours
    expect(sayDuration(Time.year, '# days')).toBe('365 days')
    expect(sayDuration(Time.year, '# weeks')).toBe('52 weeks')
    expect(sayDuration(Time.year, '# hours')).toBe('8766 hours') //a year is 365 and a quarter days, and here the quarter day shows

    //everything rounds down, so whatever is smaller than the smallest unit goes unsaid
    expect(sayDuration(2 * Time.hour - 1, '# hours')).toBe('1 hour') //a millisecond short of two hours is still one hour

    //short forms for tight spaces: a single letter never changes, and a short plural like hrs is singular for exactly 1
    expect(
      sayDuration(Time.day + 2 * Time.hour + 3 * Time.minute, '#d #h #m'),
    ).toBe('1d 2h 3m')
    expect(sayDuration(Time.hour + 5 * Time.minute, '# hr # min')).toBe(
      '1 hr 5 min',
    )
    expect(sayDuration(Time.hour + Time.minute, '# hrs # mins')).toBe(
      '1 hr 1 min',
    )
    expect(sayDuration(2 * Time.year + 3 * Time.month, '# yrs # mos')).toBe(
      '2 yrs 3 mos',
    ) //the way a profile says how long someone held a job

    //units can be written in lowercase, title case, or all capitals, and keep it
    expect(
      sayDuration(3 * Time.day + 4 * Time.hour, '# Days # Hours Left'),
    ).toBe('3 Days 4 Hours Left')
    expect(sayDuration(Time.hour, '# HOURS')).toBe('1 HOUR')

    //a bare m is minutes on its own, as in a feed, and months where only months fit, between years and the units below a month
    expect(sayDuration(5 * Time.minute, '#m ago')).toBe('5m ago')
    expect(
      sayDuration(Time.year + 2 * Time.month + 3 * Time.day, '#y #m #d'),
    ).toBe('1y 2m 3d')
    expect(
      sayDuration(
        Time.week + 2 * Time.day + 3 * Time.hour + 30 * Time.minute,
        '#w #d #h #m',
      ),
    ).toBe('1w 2d 3h 30m') //the way an issue tracker logs work

    //seconds alone can take a fraction, tenths, hundredths, or thousandths, rounded down, for a stopwatch or a benchmark
    expect(sayDuration(83456, '# minutes #.### seconds')).toBe(
      '1 minute 23.456 seconds',
    )
    expect(sayDuration(83456, '#.#s')).toBe('83.4s')

    //zeros at the start drop away with the text after them, so one template can count up a task of any length
    expect(sayDuration(32 * Time.minute + 55458, '#d #h #m #.##s')).toBe(
      '32m 55.45s',
    ) //half an hour in, from a template ready to say days
    let full = '# years, # months, # days, # hours, # minutes, #.### seconds'
    expect(sayDuration(Time.day + 250, full)).toBe(
      '1 day, 0 hours, 0 minutes, 0.250 seconds',
    ) //a zero between two others stays, rather than "1 day, 0.250 seconds", which reads like a slip
    expect(sayDuration(0, full)).toBe('0.000 seconds') //and when every unit is zero, the smallest says so

    //the smallest unit a template names is the precision it asks for, so it always shows, even on an exact amount
    expect(sayDuration(Time.minute, '#m #s')).toBe('1m 0s')
    expect(sayDuration(2 * Time.hour + 5 * Time.minute, full)).toBe(
      '2 hours, 5 minutes, 0.000 seconds',
    )

    //words around the tags are fine, since a unit only counts right after a #
    expect(sayDuration(3 * Time.hour, 'about # hours ago')).toBe(
      'about 3 hours ago',
    )
    expect(
      sayDuration(2 * Time.hour + 5 * Time.minute, '# hours and # minutes'),
    ).toBe('2 hours and 5 minutes') //the d in "and" is only a letter
    expect(sayDuration(2 * Time.hour, '# hours and # minutes')).toBe(
      '2 hours and 0 minutes',
    ) //the minutes are the precision, so they show at zero, and the sentence keeps its "and"
    expect(sayDuration(3 * Time.day, '# days of the last 10')).toBe(
      '3 days of the last 10',
    ) //and numbers of your own are only text

    //how long ago something happened is one tick subtracted from another, so keep times as ticks and say the span at the very end
    let landed = 1344230277300 //the Curiosity landing
    let later = landed + 3 * Time.day + 4 * Time.hour
    expect(sayDuration(later - landed, 'landed # days ago')).toBe(
      'landed 3 days ago',
    )

    //subtract the other way, and a negative span says its size with a minus sign in front
    expect(sayDuration(landed - later, '# days # hours')).toBe(
      '-3 days 4 hours',
    )
    expect(sayDuration(-30 * Time.second, '# minutes')).toBe('0 minutes') //unless what it says is zero
  })
  test('pitfalls of sayDuration, and what to write instead', () => {
    //a year is Time.year, 365 and a quarter days, and a month is a twelfth of that, so a calendar year of 365 days falls a quarter day short
    expect(sayDuration(365 * Time.day, '# years, # months, # days')).toBe(
      '11 months, 30 days',
    )
    expect(sayDuration(30 * Time.day, '# months, # days')).toBe('30 days') //and 30 days is a little short of a month
    expect(sayDuration(Time.year, '# years, # months, # days')).toBe(
      '1 year, 0 months, 0 days',
    ) //while a span built from Time comes out round

    //write the plural; a singular word in the template is a short tag and some letters, so it never changes
    expect(sayDuration(5 * Time.day, '# day')).toBe('5 day') //d, and then "ay"

    //on its own, a bare m is minutes, so say months alone with mo
    expect(sayDuration(3 * Time.month, '#m')).toBe('131490m')
    expect(sayDuration(3 * Time.month, '#mo')).toBe('3mo')

    //a # right before a unit's letter is a tag, so keep hashtags out of the template
    expect(sayDuration(5 * Time.hour, 'posted # hours ago #hiring')).toBe(
      'posted 5 hours ago 5hiring',
    ) //#h, and then "iring"

    //each # after the point is a digit always written, the way Excel writes 0.000, not the way it writes #.###
    expect(sayDuration(1500, '#.### seconds')).toBe('1.500 seconds')

    //a fraction only says seconds; before any other unit it's text, written as it is
    expect(sayDuration(5 * Time.minute, '#.## minutes')).toBe('#.## minutes')

    //list units from largest to smallest; out of order, each still gets its share, and each is written where it sits
    expect(
      sayDuration(2 * Time.hour + 5 * Time.minute, '# minutes # hours'),
    ).toBe('5 minutes 2 hours')

    //the smallest unit always shows, so where a trailing zero would read oddly, choose the template by the span's size, the way sayTimeRemaining does
    expect(sayDuration(27 * Time.hour, '# days # hr # min')).toBe(
      '1 day 3 hr 0 min',
    ) //a map would say "1 day 3 hr", from "# days # hr" for a trip of a day or more

    //the minus sign of a negative span goes in front of everything, words included, so choose the template by the sign
    expect(sayDuration(-3 * Time.hour, 'about # hours ago')).toBe(
      '-about 3 hours ago',
    ) //for a time still to come, write "in # hours" and say -span
  })
  test('demonstration of three shortcuts that choose their own template: time remaining, coarse, and race', () => {
    //sayTimeRemaining says one or two units, whichever matter at the span's size: two while the larger is a single digit, then one
    let remaining = [
      45 * Time.second,
      Time.minute,
      9 * Time.minute + 59 * Time.second,
      10 * Time.minute,
      Time.hour + 30 * Time.minute,
      10 * Time.hour,
      3 * Time.day - 1,
      3 * Time.day,
    ].map(sayTimeRemaining)
    expect(remaining).toEqual([
      '45s',
      '1m 0s', //the seconds are its precision here, so they show at zero
      '9m 59s',
      '10m', //ten minutes and up drops the seconds
      '1h 30m',
      '10h',
      '71h', //hours up to 71, plainer than "2d 23h" for something that finishes this week
      '3d',
    ])
    expect(sayTimeRemaining(-90 * Time.second)).toBe('-1m 30s') //overdue, with a minus sign in front

    //sayTimeRemainingCoarse rounds the seconds down to a multiple of 5, so a countdown on a page steps rather than flickers
    expect(sayTimeRemainingCoarse(37 * Time.second)).toBe('35s')
    expect(sayTimeRemainingCoarse(97 * Time.second)).toBe('1m 35s')
    expect(
      [5, 4, 3, 2, 1, 0].map((n) => sayTimeRemainingCoarse(n * Time.second)),
    ).toEqual(['5s', '4s', '3s', '2s', '1s', '0s']) //but the last five seconds count down one at a time

    //sayTimeRace says how long something took to the millisecond, the way a race is timed
    expect(sayTimeRace(5 * Time.minute + 15 * Time.second + 223)).toBe(
      `5'15"223`,
    )
    expect(sayTimeRace(7)).toBe(`0'00"007`) //seconds and milliseconds padded, so every time reads the same way
    expect(sayTimeRace(75 * Time.minute)).toBe(`75'00"000`) //the minutes take everything above them
    expect(sayTimeRace(-1234)).toBe(`-0'01"234`) //a negative span, like a split behind the leader

    //each refuses what sayDuration does: a span past the integer ceiling, or anything that isn't a number
    expect(() => sayTimeRemaining(Number.MAX_SAFE_INTEGER + 1)).toThrow()
    expect(() => sayTimeRemainingCoarse('5')).toThrow()
    expect(() => sayTimeRace(NaN)).toThrow()
  })
  test('in javascript, a span drops its fraction of a millisecond, and the say functions refuse what is not a number or a template', () => {
    //the shared vectors pin every edition alike; these are the cases only javascript and python can hand over, since rust and go take whole numbers by their types
    expect(sayDuration(1500.9, '#.###s')).toBe('1.500s') //a fraction of a millisecond drops away, as from performance.now()
    expect(sayDuration(-1500.9, '#.###s')).toBe('-1.500s') //toward zero, so the sign stays with the size
    expect(sayDuration(-0.5, '#s')).toBe('0s') //and a negative that rounds to zero has no sign
    expect(sayTimeRace(1234.5)).toBe(`0'01"234`)
    expect(hasInt(1.5) || hasInt('5') || hasInt(NaN)).toBe(false) //hasInt is strict, since it asks about an integer to carry, not one to say
    expect(() => sayDuration('5', '# seconds')).toThrow() //text isn't a number
    expect(() => sayDuration(NaN, '# seconds')).toThrow()
    expect(() => sayDuration(Infinity, '# seconds')).toThrow()
    expect(() => sayDuration(true, '# seconds')).toThrow() //nor is a boolean
    expect(() => sayDuration(0, 7)).toThrow() //a template must be text
    expect(() => sayDuration(0)).toThrow()
  })
})

//  _   _      _
// | |_(_) ___| | __
// | __| |/ __| |/ /
// | |_| | (__|   <
//  \__|_|\___|_|\_\
//

describe.skipIf(!javascriptChosen)('tick', () => {
  test('writes a tick two ways, and reads each back exactly', () => {
    // #region tick
    let t = 1344230277300 //a tick: milliseconds since 1970, the way Date.now() counts; this one is the Curiosity landing
    let text = tickToText(t) //"2012aug6.0517.57300", as short as the moment allows
    let iso = tickToIso(t) //"2012-08-06T05:17:57.300Z", always 24 characters
    let same = textToTick(text) == t && isoToTick(iso) == t //true: each form reads back to exactly t, and refuses any text its writer wouldn't produce
    let asked = hasTick(-1) //false: ask about a number from outside, like a clock or a file's modified time, before handing it to a function that takes a tick
    // #endregion tick
    expect(text).toBe('2012aug6.0517.57300')
    expect(iso).toBe('2012-08-06T05:17:57.300Z')
    expect(same).toBe(true)
    expect(asked).toBe(false)
  })
  test('says a tick for a person', () => {
    // #region saytick
    let t = 1344230277300 //the Curiosity landing
    let page = sayTick(t, 'Dddd, Mmmm D, YYYY') //"Monday, August 6, 2012": uppercase tags are the calendar
    let log = sayTick(t, 'YYYY-MM-DD hh24:mm:ss.ttt') //"2012-08-06 05:17:57.300": lowercase tags are the clock
    let clock = sayTick(t, 'h12:mm AA') //"5:17 AM"
    // #endregion saytick
    expect([page, log, clock]).toEqual([
      'Monday, August 6, 2012',
      '2012-08-06 05:17:57.300',
      '5:17 AM',
    ])
  })
  test('demonstration of saying a tick in any shape, for a page, a person, or a log', () => {
    let t = 1344230277300 //the Curiosity landing again: Monday, August 6, 2012, at 5:17:57.300 in the morning, utc

    //uppercase tags are the calendar, lowercase tags are the clock, and everything else is written as it is
    expect(sayTick(t, 'YYYY-MM-DD')).toBe('2012-08-06')
    expect(sayTick(t, 'hh24:mm:ss')).toBe('05:17:57')

    //the iso form, built from tags; T and Z aren't tags, so they pass through, and ttt is the milliseconds
    expect(sayTick(t, 'YYYY-MM-DDThh24:mm:ss.tttZ')).toBe(
      '2012-08-06T05:17:57.300Z',
    )
    expect(sayTick(t, 'YYYY-MM-DDThh24:mm:ss.tttZ')).toBe(tickToIso(t)) //the same text tickToIso writes, though only tickToIso's comes with a reader

    //for a blog or a page, with names in the case of their tags
    expect(sayTick(t, 'Dddd, Mmmm D, YYYY')).toBe('Monday, August 6, 2012')
    expect(sayTick(t, 'Mmm D, YYYY')).toBe('Aug 6, 2012')
    expect(sayTick(t, 'Ddd Mmm D')).toBe('Mon Aug 6')
    expect(sayTick(t, 'DDD MMM D')).toBe('MON AUG 6') //all capitals, for a heading or a stamp
    expect(sayTick(t, 'ddd mmm D')).toBe('mon aug 6') //and all lowercase, the way the short form writes its months

    //the twelve-hour clock, with am and pm in four forms; pairing them with h12 or hh12 is up to the template
    expect(sayTick(t, 'h12:mm AA')).toBe('5:17 AM')
    expect(sayTick(t, 'hh12:mm aa')).toBe('05:17 am')
    expect(sayTick(t, 'h12:mmA')).toBe('5:17A')
    expect(sayTick(t, 'h12:mma')).toBe('5:17a')

    //for a log file, every field padded so the lines align and sort as text, with the fraction as fine as you need
    expect(sayTick(t, 'YYYY-MM-DD hh24:mm:ss.ttt')).toBe(
      '2012-08-06 05:17:57.300',
    )
    expect(sayTick(t, 'YYYY-MM-DD hh24:mm:ss.t')).toBe('2012-08-06 05:17:57.3') //t, tt, and ttt are tenths, hundredths, and thousandths, rounded down
    expect(sayTick(t, 'YYYYMMDD-hh24mmss')).toBe('20120806-051757') //a file name that sorts, with no colons for the file systems that refuse them

    //numbers in the order different places write a date; the tags neither know nor care whose order it is
    expect(sayTick(t, 'M/D/YYYY')).toBe('8/6/2012')
    expect(sayTick(t, 'DD.MM.YYYY')).toBe('06.08.2012')

    //sayTick writes the tick's fields as they fall in utc, so for the clock on someone's wall, shift the tick by their offset first
    let pacific = t - 7 * Time.hour //pacific daylight time is seven hours behind utc; t - new Date(t).getTimezoneOffset() * Time.minute shifts to this machine's own
    expect(sayTick(pacific, 'Dddd, Mmmm D, h12:mm AA')).toBe(
      'Sunday, August 5, 10:17 PM',
    ) //back across midnight, to the evening before

    //the template is read once, left to right, taking the longest tag at each position
    expect(sayTick(t, 'YYYYY')).toBe('2012Y') //YYYY, and then a Y that isn't a tag
    expect(sayTick(t, 'MMMMM')).toBe('AUGUST8') //MMMM, and then M
    expect(sayTick(t, 'h12:m')).toBe('5:m') //the minute is mm and always two digits, so a lone m is only a letter; five past is :05, never :5
    expect(sayTick(t, 'h12:mm a.m.')).toBe('5:17 a.m.') //which lets a.m. and p.m. keep their periods, the way the associated press writes them
    expect(sayTick(t, 'MMM')).toBe('AUG') //and a value is never read again, so the A in AUG stays a letter rather than becoming AM or PM

    //keep words out of the template: say the date with a small one, and put what it writes into the text around it
    expect(`Posted ${sayTick(t, 'Mmm D')} at ${sayTick(t, 'h12:mm AA')}`).toBe(
      'Posted Aug 6 at 5:17 AM',
    )
    expect(sayTick(t, 'Posted Mmm D at h12:mm AA')).toBe(
      'Pos3ed Aug 6 a3 5:17 AM',
    ) //because written into the template, the t in "Posted" and the a and t in "at" are tags
  })
  test('in javascript, sayTick drops a fraction of a millisecond, keeps its names frozen, and refuses what is not a number or a template', () => {
    expect(sayTick(1344230277300.7, 'ss.ttt')).toBe('57.300') //a fraction of a millisecond drops down to the millisecond the moment falls in
    expect(() => sayTick(-0.5, 'YYYY')).toThrow() //which, half a millisecond before 1970, is not a tick
    expect(() => sayTick('5', 'YYYY')).toThrow() //text isn't a number
    expect(() => sayTick(NaN, 'YYYY')).toThrow()
    expect(() => sayTick(true, 'YYYY')).toThrow()
    expect(() => sayTick(0, 7)).toThrow() //a template must be text
    expect(() => sayTick(0)).toThrow()
    expect(Object.isFrozen(monthNames) && Object.isFrozen(dayNames)).toBe(true) //so a caller can't change them under sayTick and the short form
    expect(monthNames[8].slice(0, 3).toLowerCase()).toBe('sep') //the first three letters are the short name, which the short form writes in lowercase
  })
  test('sayTick agrees with tickToIso on every tick the shared vectors pin', () => {
    //the ticks the tick vectors write or read: unit boundaries, leap days, and both edges of the range
    let ticks = new Set()
    for (let c of vectors) {
      if (c.throws) continue //a refusal's input isn't a tick
      if (c.function == 'tickToIso' || c.function == 'tickToText')
        ticks.add(c.input[0])
      if (c.function == 'isoToTick' || c.function == 'textToTick')
        ticks.add(c.output)
    }
    expect(ticks.has(0) && ticks.has(253402300799999)).toBe(true) //among them both edges of the range, so the check below reaches them
    for (let t of ticks)
      expect(sayTick(t, 'YYYY-MM-DDThh24:mm:ss.tttZ'), String(t)).toBe(
        tickToIso(t),
      ) //built from tags, the iso form comes out the same as the platform writes it
  })
})

//  _
// | |_ __ _  __ _
// | __/ _` |/ _` |
// | || (_| | (_| |
//  \__\__,_|\__, |
//           |___/

describe.skipIf(!javascriptChosen)('Tag', () => {
  test('makes a tag, and recognizes it', () => {
    // #region tag
    let t = Tag() //a new tag, like "AgKxru95C7jFp5iPuK9O7": 21 letters and numerals from the operating system's random source
    let valid = hasTag(t) //true
    let invalid = hasTag('not a tag') //false
    let same = checkTag(t) //returns t, or throws because the caller handed it something that isn't a tag
    // #endregion tag
    expect(t.length).toBe(tagLength)
    expect(valid).toBe(true)
    expect(invalid).toBe(false)
    expect(same).toBe(t)
  })
})

//      _       _
//   __| | __ _| |_ __ _
//  / _` |/ _` | __/ _` |
// | (_| | (_| | || (_| |
//  \__,_|\__,_|\__\__,_|
//

describe.skipIf(!javascriptChosen)('Data', () => {
  test('holds bytes, and speaks every form they travel in', () => {
    // #region data
    let d = Data({ text: 'hello' }) //bytes, from text
    let forms = [d.base16(), d.base32(), d.base62(), d.base64()] //out in any form: 68656c6c6f, NBSWY3DP, Q6LhR6z, aGVsbG8=
    let back = Data({ base62: 'Q6LhR6z' }).text() //and back from any, each read exactly: "hello"
    let part = d.clipView(1, 3).text() //part of it, without copying: "el"
    let b = Bin(6) //a set number of bytes, filled a byte or a Data at a time
    b.add(d)
    b.add(0x21)
    let whole = b.data().text() //"hello!"
    // #endregion data
    expect(forms).toEqual(['68656c6c6f', 'NBSWY3DP', 'Q6LhR6z', 'aGVsbG8='])
    expect(back).toBe('hello')
    expect(part).toBe('el')
    expect(whole).toBe('hello!')
  })
  test('demonstration of javascript arrays and buffers through clipping while intentionally choosing viewing or copying', () => {
    let text = 'ABCDEFGHIJKL' //12 letters
    let encoder = new TextEncoder()
    let a = encoder.encode(text) //turn the text into bytes; makes a buffer and returns an array that looks at the buffer
    let b = a.buffer //get a reference to the underlying buffer
    expect(a instanceof Uint8Array).toBe(true)
    expect(b instanceof ArrayBuffer).toBe(true)

    expect(a.length).toBe(12)
    expect(a.byteLength).toBe(12) //arrays have .length and .byteLength
    expect(b.byteLength).toBe(12) //buffers only have .byteLength
    let { start, end, length } = { start: 3, end: 6, length: 3 } //most js apis take (start, end) but some take (start, length)

    let a1 = a.subarray(start, end) //(1) array, view
    let a2 = a.slice(start, end) //(2) array, copy: array slice makes a copy
    let a3 = new Uint8Array(b, start, length) //(3) buffer, view: note length here, not end
    let b4 = b.slice(start, end) //(4) buffer, copy: buffer slice makes a copy
    let a4 = new Uint8Array(b4) //and the array constructor clips a new view around the given buffer

    expect(a1 instanceof Uint8Array).toBe(true)
    expect(a2 instanceof Uint8Array).toBe(true)
    expect(a3 instanceof Uint8Array).toBe(true)
    expect(a4 instanceof Uint8Array).toBe(true) //array methods subarray and slice and array constructor make arrays
    expect(b4 instanceof ArrayBuffer).toBe(true) //buffer slice produces a buffer
    expect(Data({ array: a1 }).text()).toBe('DEF') //and a Data looks at any of them the same way
    expect(Data({ buffer: b4 }).text()).toBe('DEF')
  })

  //the tests below pin what a Data is as a javascript object, which the shared vectors can't see, since json carries values and not memory
  test('looks at the array or buffer it was given without copying, and clipView looks where clipCopy copies', () => {
    let a = new Uint8Array([0x41, 0x42, 0x43, 0x44, 0x45, 0x46]) //ABCDEF
    let d = Data({ array: a })
    expect(d.array()).toBe(a) //the very array it was given
    let view = d.clipView(1, 5) //BCDE, looking at a
    let copy = d.clipCopy(1, 5) //BCDE, in bytes of its own
    let inner = view.clipView(1, 3) //CD, since a clip of a clip counts from the clip
    expect(inner.base16()).toBe('4344')

    a[2] = 0x63 //change the C beneath all of them to c
    expect(d.get(2)).toBe(0x63)
    expect(view.get(1)).toBe(0x63) //a view sees the change
    expect(inner.get(0)).toBe(0x63) //and so does a view of a view
    expect(copy.get(1)).toBe(0x43) //a copy doesn't
    inner.array()[1] = 0x64 //and a write through a view reaches the bytes beneath
    expect(a[3]).toBe(0x64)

    let buffer = new ArrayBuffer(3)
    let b = Data({ buffer }) //a Data over all of a buffer, also without copying
    new Uint8Array(buffer).set([0x78, 0x79, 0x7a])
    expect(b.text()).toBe('xyz')
    expect(b.array().buffer).toBe(buffer)
  })
  test('works out each form once and keeps it, and keeps the form it was made from as given', () => {
    let a = new Uint8Array([0x61, 0x62, 0x63]) //abc
    let d = Data({ array: a })
    expect(d.base16()).toBe('616263') //worked out on first ask, and kept
    a[0] = 0x7a //change the a beneath it to z
    expect(d.get(0)).toBe(0x7a) //the bytes are what they are now
    expect(d.base16()).toBe('616263') //but a form already written stays as it was
    expect(d.text()).toBe('zbc') //and a form not yet asked for is written from the bytes as they are now
    expect(d.base62()).toBe(Data({ text: 'zbc' }).base62())

    let t = Data({ text: 'abc' })
    t.array()[0] = 0x7a
    expect(t.text()).toBe('abc') //the text it was made from, kept as given
    expect(t.base16()).toBe('7a6263') //while the other forms follow the bytes
    let s = Data({ base16: '616263' })
    s.array()[0] = 0x7a
    expect(s.base16()).toBe('616263') //the same for a form it was read from
    expect(s.text()).toBe('zbc')
  })
  test('made from several forms at once, takes the first in a fixed order', () => {
    //one byte in each form, each a different byte, so the answer shows which form won
    let forms = {
      array: new Uint8Array([0x01]),
      buffer: new Uint8Array([0x02]).buffer,
      text: '\u0003',
      base16: '04',
      base32: Data({ base16: '05' }).base32(),
      base62: Data({ base16: '06' }).base62(),
      base64: Data({ base16: '07' }).base64(),
      random: 8, //eight random bytes, told apart by their size
    }
    let names = Object.keys(forms)
    for (let i = 0; i < names.length; i++) {
      let p = Object.fromEntries(
        names.slice(i).map((name) => [name, forms[name]]),
      ) //this form and every one after it
      let d = Data(p)
      if (names[i] == 'random') expect(d.size()).toBe(8)
      else expect(d.base16(), names[i]).toBe('0' + (i + 1))
    }
  })
  test('in javascript, refuses a form of the wrong type, or none', async () => {
    let refused = [
      {}, //no form at all
      { array: [0x41] }, //a plain array, where a Uint8Array belongs
      { buffer: new Uint8Array(1) }, //an array, where a buffer belongs
      { text: 5 },
      { base16: 0x41 },
      { random: '3' },
      { random: -1 },
      { random: 1.5 },
      { random: null },
      { text: '\ud800' }, //a lone surrogate, half of a pair, which has no UTF-8
    ]
    for (let p of refused) expect(() => Data(p), JSON.stringify(p)).toThrow()
    expect(() => Data()).toThrow()
    expect(() => Data({ base16: '41' }).get(0.5)).toThrow() //and an index that isn't a whole number
    await expect(sha256('abc')).rejects.toThrow() //and the functions that take a Data refuse anything else
    expect(() => Bin(3).add('abc')).toThrow()
  })
  test('fills a large random request in steps', () => {
    let size = 3 * 65536 + 16 //getRandomValues fills at most 65536 bytes a call, so this takes four
    let d = Data({ random: size })
    expect(d.size()).toBe(size)
    for (let i = 0; i < size; i += 65536) {
      let step = d.clipView(i, Math.min(i + 65536, size)).array()
      expect(step.some((b) => b != 0)).toBe(true) //each step got its bytes, since a step of random bytes that are all zero won't happen
    }
  })
  test('says its type, and keeps its bytes out of json', () => {
    let d = Data({ text: 'secret' })
    expect(d.type).toBe('Data')
    expect(JSON.stringify(d)).toBe('{"type":"Data"}')
  })
})

//  _               _
// | |__   __ _ ___| |__
// | '_ \ / _` / __| '_ \
// | | | | (_| \__ \ | | |
// |_| |_|\__,_|___/_| |_|
//

//the three hashes by name, with the function that takes all the bytes at once and the hasher that takes a piece at a time
const hashPairs = [
  ['sha1', sha1, Sha1],
  ['sha256', sha256, Sha256],
  ['blake3', blake3, Blake3],
]

describe.skipIf(!javascriptChosen)('hash', () => {
  test('hashes bytes all at once, and a file a piece at a time', async () => {
    // #region hash
    let d = await sha256(Data({ text: 'hello' })) //all at once, for bytes already in memory: a Data of 32 bytes, out in any form
    let file = new Blob(['hello']) //or a File the user chose, of any size
    let hashers = [Sha1(), Sha256(), Blake3()] //or a piece at a time, all three from one read of the file
    let reader = file.stream().getReader()
    while (true) {
      let { done, value } = await reader.read()
      if (done) break
      for (let h of hashers) h.update(Data({ array: value })) //a Data looks at each read without copying it
    }
    let [d1, d256, db3] = hashers.map((h) => h.digest())
    // #endregion hash
    expect(d.base16()).toBe(
      '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824',
    )
    expect(d1.base16()).toBe('aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d')
    expect(d256.base16()).toBe(d.base16())
    expect(db3.base16()).toBe(
      'ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f',
    )
    expect(await hashText('hello')).toBe(
      (await sha256(Data({ text: 'hello' }))).base32(),
    ) //and the shortcut for text, in base32
  })
  test("digest doesn't finish a hasher, which keeps going for the hash of everything so far", async () => {
    let a = Data({ text: 'a' })
    let ab = Data({ text: 'ab' })
    for (let [name, all, Hasher] of hashPairs) {
      let h = Hasher()
      h.update(a)
      expect(h.digest().base16(), name).toBe((await all(a)).base16())
      expect(h.digest().base16(), name).toBe((await all(a)).base16()) //a second digest is the same answer
      h.update(ab.clipView(1, 2))
      expect(h.digest().base16(), name).toBe((await all(ab)).base16()) //and the hasher took more after both
    }
  })
})

describe.skipIf(!javascriptChosen)('fuji hashes', () => {
  test('hash a file quickly by its tips, and wholly by its pieces', async () => {
    // #region fuji
    let file = new Blob(['hello']) //or a File the user chose, of any size
    let { tipHash } = await hashFile({ file, size: file.size }) //the tip hash, from a few small slices of the file
    let { pieceHash } = await hashStream({
      stream: file.stream(),
      size: file.size,
    }) //the piece hash, from one read of all of it, which gives the tip hash too

    let hasher = FujiHasher({ size: 5 }) //or feed one the pieces as they arrive, like the chunks of an upload
    await hasher.update(Data({ text: 'he' }))
    await hasher.update(Data({ text: 'llo' }))
    let both = await hasher.digest() //both hashes, the same as from the stream
    // #endregion fuji
    expect(both.tipHash.base32()).toBe(tipHash.base32())
    expect(both.pieceHash.base32()).toBe(pieceHash.base32())
    expect(tipHash.base32()).toBe(
      'BSOEFHWYKUFE2ZEYFGKAE2X4IZXTXXCDJZQ2YRGKLMAETUJDTIHQ',
    )
    expect(pieceHash.base32()).toBe(
      'AD4G5U6L4LJC4DYUIUSFRIYHH5KMRVHCLOQQAKAPNNPFCRAUIV3Q',
    )
    expect(tipHash.base32()).toBe(
      await hashText('Fuji.Tips.SHA256.4KiB.5.hello'),
    ) //confirm matching results when performing the hashing by hand ✏️
  })
})

//              _   _ _
//   ___  _   _| |_| (_)_ __   ___
//  / _ \| | | | __| | | '_ \ / _ \
// | (_) | |_| | |_| | | | | |  __/
//  \___/ \__,_|\__|_|_|_| |_|\___|
//

describe.skipIf(!javascriptChosen)('Outline', () => {
  test('main usage: build from a template literal, read through the navigation API, round-trip through text and binary', () => {
    let o = textToOutline(deindent`
      orchard:
        tree:"apple"
        season:"autumn"
        basket:
          color:"red"
          count:"12"
    `)

    //n navigates to a named outline and throws if missing; m makes it if missing then navigates
    expect(o.n('tree').value().text()).toBe('apple')
    expect(o.n('basket').n('color').value().text()).toBe('red')
    expect(() => o.n('missing')).toThrow() //thrown, not silent
    o.m('picker').value(Data({ text: 'bear' })) //create and set in one step
    expect(o.n('picker').value().text()).toBe('bear')

    //round-trip through both serialization formats, confirming identity
    expect(textToOutline(o.text()).data().base16()).toBe(o.data().base16()) //text → outline → same bytes
    expect(dataToOutline(o.data()).text()).toBe(o.text()) //binary → outline → same text
  })
  test('binary form is deterministic: the same information always produces the same bytes, regardless of insert order', () => {
    //two outlines constructed differently, but with the same information
    let a = Outline('garden')
    a.m('beds')
      .m('flowers')
      .add(Outline('name', Data({ text: 'marigold' })))
    a.m('beds')
      .m('herbs')
      .add(Outline('name', Data({ text: 'basil' })))
    a.add(Outline('season', Data({ text: 'spring' })))

    let b = textToOutline(deindent`
      garden:
        season:"spring"
        beds:
          herbs:
            name:"basil"
          flowers:
            name:"marigold"
    `)

    //identical bytes means the hash of an outline is a stable identity for its contents
    expect(a.data().base16()).toBe(b.data().base16())
  })
  test('contents: add, has, list, remove, clear', () => {
    let o = Outline('pantry')

    //add takes an Outline, a name string, or Data; multiple outlines can share a name, and unnamed outlines are supported
    o.add(Outline('jar', Data({ text: 'honey' })))
    o.add(Outline('jar', Data({ text: 'jam' })))
    o.add('crock') //string shorthand: adds an outline with this name and no value
    o.add(Data({ text: 'salt' })) //data shorthand: adds an unnamed outline with this value
    o.add(Data({ text: 'pepper' }))

    expect(o.length()).toBe(5)
    expect(o.has('jar')).toBe(true)
    expect(o.has('jug')).toBe(false)

    //list returns all contained outlines with a given name, or all unnamed if no name passed
    expect(o.list('jar').length).toBe(2)
    expect(o.list().length).toBe(2) //the two unnamed

    //falsy lookups all mean "look for blank name"
    expect(o.list(null).length).toBe(2)
    expect(o.list('').length).toBe(2)

    o.remove('jar')
    expect(o.length()).toBe(3)
    expect(o.list('jar').length).toBe(0)

    o.clear() //discards value and contents, but keeps the name
    expect(o.length()).toBe(0)
    expect(o.name()).toBe('pantry')
  })
  test('validation: names must be [a-z0-9], values must be Data or null', () => {
    //names are restricted to lowercase letters and digits; no ambiguity like the HTTP header case-folding mess
    expect(() => Outline('Key1')).toThrow() //uppercase
    expect(() => Outline('key-1')).toThrow() //punctuation
    expect(() => Outline().name(' ')).toThrow() //whitespace
    expect(() => Outline().n('Key1')).toThrow() //lookups validate too, so you get a loud error instead of a silent "not found"

    //values are Data, and a Data of no bytes is no value; null clears to it
    let o = Outline('x')
    expect(() => o.value(42)).toThrow()
    expect(() => o.value('just a string')).toThrow()
    o.value(Data({ text: 'something' }))
    expect(o.value().text()).toBe('something')
    o.value(null) //null always valid, clears to no value
    expect(o.value().size()).toBe(0)
  })
  test('textToOutline indent algorithm: root at zero, child indents can be any width but must be consistent among siblings', () => {
    //dots stand in for spaces so the indentation is visible at a glance, immune to editor auto-formatting
    let dots = (s) => s.replace(/\./g, ' ')
    let good = (s) => expect(() => textToOutline(dots(s))).not.toThrow()
    let poor = (s) => expect(() => textToOutline(dots(s))).toThrow()

    good(deindent`
      a:
      ..b:
      ..c:
    `) //two children at the same indent

    good(deindent`
      a:
      ....b:
      ....c:
    `) //4-space works too--it's about consistency, not width

    good(deindent`
      a:
      ....b:
      ......c:
      ......e:
      ..........f:
      ..........g:
      ....d:
    `) //deeper levels can use different widths, each parent's contents just need to agree among themselves

    poor(deindent`
      ..a:
    `) //root must be at zero indent

    poor(deindent`
      a:
      b:
    `) //two roots at zero indent is two outlines, not one

    poor(deindent`
      a:
      ..b:
      c:
    `) //outdenting back to zero is also two roots

    poor(deindent`
      a:
      ..b:
      .c:
    `) //a child at an indent that doesn't match its siblings is ambiguous
  })
  test("textToOutline rejects malformed structure; quoted encoding in values throws through quotedToData's own validation", () => {
    //structural errors
    expect(() => textToOutline('')).toThrow() //no outline to parse
    expect(() => textToOutline('nocolon')).toThrow() //line without a colon--can't split name from value
    expect(() =>
      textToOutline(deindent`
        a:
          b:
        c:
      `),
    ).toThrow() //valid tree followed by a second root

    //a blank-everywhere line--no name, no value, no contents--is the minimum valid outline
    let o = textToOutline(':')
    expect(o.name() == '' && o.value().size() == 0 && o.length() == 0).toBe(
      true,
    )
  })
  test('textToOutline accepts trailing whitespace and # comments on each line, so pasted or hand-edited text tolerates reasonable sloppiness', () => {
    //dots stand in for spaces so the trailing whitespace is visible
    let dots = (s) => s.replace(/\./g, ' ')
    let good = (s) => expect(() => textToOutline(dots(s))).not.toThrow()
    let poor = (s) => expect(() => textToOutline(dots(s))).toThrow()

    //baseline: no trailing content
    good(deindent`
      a:
      ..b:"7"
    `)

    //trailing whitespace after the value
    good(deindent`
      a:
      ..b:"7".
    `)

    //# starts a comment; everything from # to end of line is ignored
    good(deindent`
      a:
      ..b:"7"#comment
    `)

    //whitespace between the value and the comment is fine
    good(deindent`
      a:
      ..b:"7".#comment
    `)

    //# inside a quoted value is not a comment marker
    good(deindent`
      a:
      ..b:"room #9"
    `)

    //trailing content that is neither whitespace nor a comment is rejected
    poor(deindent`
      a:
      ..b:"7".extra
    `)
  })
  test('span encoding: variable-length integer used for size prefixes in the binary form', () => {
    function both(n, base16) {
      expect(spanEncode(n).base16()).toBe(base16) //encode
      expect(spanDecode(Data({ base16 }), 0)).toEqual({
        value: n,
        bytesRead: base16.length / 2,
      }) //decode
    }

    //boundaries at each byte threshold--7 bits per byte, up to 4 bytes
    both(0, '00')
    both(127, '7f') //1 byte: 0 through 127
    both(128, '8001')
    both(16383, 'ff7f') //2 bytes: 128 through 16383
    both(16384, '808001')
    both(2097151, 'ffff7f') //3 bytes: 16384 through 2097151
    both(2097152, '80808001')
    both(268435455, 'ffffff7f') //4 bytes: 2097152 through 268435455
    expect(spanDecode(Data({ base16: '00ff7f' }), 1)).toEqual({
      value: 16383,
      bytesRead: 2,
    }) //and from partway into the bytes, the way a reader moves through them

    //encode rejects out of range
    expect(() => spanEncode(-1)).toThrow()
    expect(() => spanEncode(268435456)).toThrow() //one past the 4-byte cap

    //decode rejects non-canonical and truncated input
    expect(() => spanDecode(Data({ base16: '8000' }), 0)).toThrow() //non-canonical: two bytes encoding the value 0, canonical is one byte '00'
    expect(() => spanDecode(Data({ base16: '80' }), 0)).toThrow() //truncated: continuation bit set with no next byte
    expect(() => spanDecode(Data({ base16: '80808080' }), 0)).toThrow() //four continuation bytes and no end, past the 4-byte cap
  })
  test('names: has and check ask about a name from outside, and only undefined, null, and blank mean no name', () => {
    expect(hasOutlineName('key1')).toBe(true)
    expect(hasOutlineName('')).toBe(true) //blank is a name, the blank one
    expect(hasOutlineName('Key1')).toBe(false)
    expect(hasOutlineName('key 1')).toBe(false)
    expect(hasOutlineName(5)).toBe(false)
    expect(hasOutlineName(null)).toBe(false) //null isn't a name, though a lookup takes it to mean the blank one
    expect(checkOutlineName('key1')).toBe('key1')
    expect(() => checkOutlineName('Key1')).toThrow()

    let o = Outline('x')
    o.add(Data({ text: 'unnamed' }))
    expect(o.list(undefined).length).toBe(1) //undefined and null mean the blank name
    expect(o.list(null).length).toBe(1)
    expect(() => o.list(0)).toThrow() //and 0 and false, though falsy, don't
    expect(() => o.list(false)).toThrow()
  })
  test('values: no value is a Data of no bytes, the same in memory, in text, and in binary', () => {
    let o = Outline('x')
    expect(o.value().size()).toBe(0) //a new outline has no value, which is an empty Data, never null
    o.value(Data({ base16: '' })) //and an empty Data set as a value is no value
    expect(o.text()).toBe('x:\n')
    expect(o.data().base16()).toBe('01780000') //the name's span 01 and the name x, then 00 for no value and 00 for no contents
    expect(dataToOutline(o.data()).value().size()).toBe(0) //and it reads back the same way
    expect(textToOutline('x:').value().size()).toBe(0)
  })
  test('writers sort a copy, so text and bytes come out canonical and the outline stays as it was built', () => {
    let o = Outline('x')
    o.add('b')
    o.add('a')
    expect(o.text()).toBe('x:\n  a:\n  b:\n') //the text comes out sorted
    expect(o.data().base16()).toBe('017800080161000001620000') //and so do the bytes: x, then a, then b
    expect(o.get(0).name()).toBe('b') //while the outline keeps the order it was built in
    expect(o.get(1).name()).toBe('a')
    o.sort() //and sorts in place only when asked
    expect(o.get(0).name()).toBe('a')
  })
  test('the binary reader refuses contents out of order, so the same information is always the same bytes', () => {
    expect(
      dataToOutline(Data({ base16: '017800080161000001620000' })).text(),
    ).toBe('x:\n  a:\n  b:\n') //x, holding a and then b
    expect(() =>
      dataToOutline(Data({ base16: '017800080162000001610000' })),
    ).toThrow() //x, holding b and then a, out of order
    expect(
      dataToOutline(Data({ base16: '017800080161000001610000' })).length(),
    ).toBe(2) //and two the same are fine, since a tie isn't out of order
  })
  test('binary form: arbitrary trees round-trip through dataToOutline and back, and trailing or malformed input is refused', () => {
    let roundTrip = (o) =>
      expect(dataToOutline(o.data()).data().base16()).toBe(o.data().base16())

    roundTrip(Outline()) //empty outline
    roundTrip(Outline('leaf', Data({ text: 'value' }))) //simple leaf
    let deep = Outline('a')
    deep
      .m('b')
      .m('c')
      .m('d')
      .value(Data({ text: 'deep' }))
    roundTrip(deep)
    let mix = Outline('mix')
    mix.add(Data({ base16: '01' }))
    mix.add(Data({ base16: '02' }))
    mix.add(Outline('tag', Data({ text: 'label' })))
    roundTrip(mix) //mixed named and unnamed contents

    expect(() => dataToOutline(Data({ base16: '000000ff' }))).toThrow() //trailing byte after a valid empty outline
  })
  test('serialization rejects cycles with a clear error, not an unhelpful RangeError from stack exhaustion', () => {
    //require a clean throw, not a stack overflow--the point is to give the caller a meaningful error
    let rejectsCycle = (f) => expect(f).toThrow(/appears twice in its tree/)

    //simplest cycle: an outline that contains itself
    let self = Outline('self')
    self.add(self)
    rejectsCycle(() => self.data())
    rejectsCycle(() => self.text())

    //longer cycle: two outlines containing each other
    let a = Outline('a')
    let b = Outline('b')
    a.add(b)
    b.add(a)
    rejectsCycle(() => a.data())
    rejectsCycle(() => a.text())

    //DAG (the same outline instance referenced from two sibling subtrees) is also rejected--the text form can't express sharing, so the binary form won't either
    let shared = Outline('shared', Data({ text: 'value' }))
    let dag = Outline('dag')
    dag.m('left').add(shared)
    dag.m('right').add(shared)
    rejectsCycle(() => dag.data())
    rejectsCycle(() => dag.text())
  })
})

//                    _           _                            _ _
//   __ _ _   _  ___ | |_ ___  __| |   ___ _ __   ___ ___   __| (_)_ __   __ _
//  / _` | | | |/ _ \| __/ _ \/ _` |  / _ \ '_ \ / __/ _ \ / _` | | '_ \ / _` |
// | (_| | |_| | (_) | ||  __/ (_| | |  __/ | | | (_| (_) | (_| | | | | | (_| |
//  \__, |\__,_|\___/ \__\___|\__,_|  \___|_| |_|\___\___/ \__,_|_|_| |_|\__, |
//     |_|                                                               |___/

describe.skipIf(!javascriptChosen)('quoted encoding', () => {
  test('demonstration and round trips', () => {
    expect(dataToQuoted(Data({ base16: '' }))).toBe('') //no bytes at all are blank quoted encoding
    expect(quotedToData('').size()).toBe(0) //and blank quoted encoding is no bytes

    expect(quotedToData('48656c6c6f').text()).toBe('Hello')
    expect(quotedToData('48"ello"').text()).toBe('Hello') //multiple different valid encoded forms decode to the same data, unlike the encoding formats built into Data, which are all guaranteed and checked to be round-trip one-to-one
    //remaining tests use examples which happen to be round trip-able
    function f(s, q) {
      expect(dataToQuoted(Data({ text: s }))).toBe(q)
      expect(quotedToData(q).text()).toBe(s)
    }

    f('hello', '"hello"') //pure text gets quoted
    f('a', '"a"')
    f(' ', '" "')

    f('A\0', '"A"00') //text with terminators
    f('AB\0', '"AB"00')
    f(`Hello\r\n`, '"Hello"0d0a') //retro Windows-style
    f('Hello\0', '"Hello"00')
    f('\tIndented', '09"Indented"')

    f('"', '22')
    f('"hello"', '22"hello"22')
    f(`The quote " character\n`, '"The quote "22" character"0a') //intentional design choice picking " because it looks like 22 ;)
  })
  test('blank describes nothing, and text that is not quoted encoding is refused', () => {
    expect(dataToQuoted(Data({ base16: '' }))).toBe('') //blank is valid quoted encoding for zero bytes,
    expect(quotedToData('').size()).toBe(0) //and quotedToData returns a Data of no bytes to indicate nothing
    expect(() => dataToQuoted(null)).toThrow() //and null isn't a Data

    expect(() => quotedToData('00ff0g')).toThrow() //g is not valid base16
    expect(quotedToData('"x"00"y"00"z"').base16()).toBe('780079007a') //this is fine
    expect(() => quotedToData('"x"00"y"00"z')).toThrow() //missing a quote
    expect(() => quotedToData('x"00"y"00"z"')).toThrow()
    expect(() => quotedToData('g')).toThrow() //single non-base16 character
    expect(() => quotedToData('000')).toThrow() //odd number of base16 digits
    expect(() => quotedToData('"中文"')).toThrow() //only ASCII allowed inside quotes
    expect(() => quotedToData('"Hello\n"')).toThrow()
    expect(() => quotedToData('0D0A')).toThrow() //uppercase base16 rejected, lowercase only
    expect(() => quotedToData(5)).toThrow() //and something that isn't a string at all
  })
})

//           _ _ _   _
//   ___  __| (_) |_(_) ___  _ __  ___
//  / _ \/ _` | | __| |/ _ \| '_ \/ __|
// |  __/ (_| | | |_| | (_) | | | \__ \
//  \___|\__,_|_|\__|_|\___/|_| |_|___/
//

//the javascript edition in-process alongside the python, rust, and go drivers, so one test body runs against all four
describe('editions', () => {
  let editions
  beforeAll(async () => {
    editions = await startEditions()
  }, 180_000) //the first run compiles the rust and go drivers
  afterAll(() => stopEditions(editions))

  describe('vectors, every edition', () => {
    for (let c of vectors) {
      test(describeVector(c), async () => {
        for (let [name, e] of Object.entries(editions)) {
          if (c.throws) await expectRefused(e[c.function](...c.input), name)
          else expect(await e[c.function](...c.input), name).toEqual(c.output)
        }
      })
    }
  })

  //random bytes can't be pinned by a vector either, past their size; this checks that every edition's are new each time
  describe('random Data, every edition', () => {
    test('is as many bytes as asked for, and never repeats', async () => {
      for (let [name, e] of Object.entries(editions)) {
        let seen = new Set()
        for (let i = 0; i < 100; i++) {
          let s = await e.Data({ random: 16 }, 'base16')
          expect(s, name).toMatch(/^[0-9a-f]{32}$/)
          expect(seen.has(s), name).toBe(false)
          seen.add(s)
        }
      }
    })
  })

  //a fraction of a millisecond, and a boolean, are values only javascript and python can hand over, since rust and go take whole numbers by their types; so these can't be vectors
  let doubles = ['javascript', 'python'].filter((name) => chosen.includes(name)) //of those two, the ones this run drives
  describe.skipIf(doubles.length == 0)(
    'say functions, the editions that take a double',
    () => {
      test('drop a fraction of a millisecond, and refuse a boolean', async () => {
        for (let name of doubles) {
          let e = editions[name]
          expect(await e.sayTick(1344230277300.7, 'ss.ttt'), name).toBe(
            '57.300',
          ) //down to the millisecond the moment falls in
          expect(await e.sayDuration(-1500.9, '#.###s'), name).toBe('-1.500s') //toward zero, so the sign stays with the size
          expect(await e.sayDuration(-0.5, '#s'), name).toBe('0s') //and a negative that rounds to zero has no sign
          expect(await e.sayTimeRace(1234.5), name).toBe(`0'01"234`)
          await expectRefused(e.sayTick(-0.5, 'YYYY'), name) //half a millisecond before 1970 is not a tick
          await expectRefused(e.sayTick(true, 'YYYY'), name) //a boolean is not a number, though python's bool is an int
          await expectRefused(e.sayDuration(true, '#s'), name)
          expect(await e.hasInt(true), name).toBe(false)
          expect(await e.hasInt(1.5), name).toBe(false) //hasInt stays strict, since it asks about an integer to carry, not one to say
          await expectRefused(e.saySize4(true), name) //and saySize4 refuses a boolean along with the fraction the vectors pin, since a fraction of a millisecond is a clock's real precision, while a fraction of a byte is a mistake
        }
      })
    },
  )

  //the vectors stay within the integer ceiling, so the refusals just past it are scripted; 2^53 is the one integer past the ceiling a double still holds exactly, so every edition reads it the same
  describe('the integer ceiling, every edition', () => {
    test('is refused one past it, either way', async () => {
      let past = 2 ** 53
      for (let [name, e] of Object.entries(editions)) {
        expect(await e.hasInt(past), name).toBe(false)
        expect(await e.hasInt(-past), name).toBe(false)
        await expectRefused(e.checkInt(past), name)
        await expectRefused(e.checkInt(-past), name)
        await expectRefused(e.sayDuration(past, '# seconds'), name)
        await expectRefused(e.sayDuration(-past, '# seconds'), name)
        await expectRefused(e.sayTimeRemaining(past), name)
        await expectRefused(e.sayTimeRemainingCoarse(-past), name)
        await expectRefused(e.sayTimeRace(past), name)
      }
    })
  })

  //Tag is random, so the vectors can't pin it; these scripted tests check what every edition's tags have in common
  describe('Tag, every edition', () => {
    test('is 21 letters and numerals, and never repeats', async () => {
      for (let [name, e] of Object.entries(editions)) {
        let seen = new Set()
        for (let i = 0; i < 100; i++) {
          let t = await e.Tag()
          expect(t, name).toMatch(/^[0-9A-Za-z]{21}$/)
          expect(seen.has(t), name).toBe(false)
          seen.add(t)
        }
      }
    })
    test('a tag from any edition validates in every edition', async () => {
      for (let [maker, m] of Object.entries(editions)) {
        let t = await m.Tag()
        for (let [checker, c] of Object.entries(editions)) {
          let pair = `${maker} made it, ${checker} checked it`
          expect(await c.hasTag(t), pair).toBe(true)
          expect(await c.checkTag(t), pair).toBe(t)
        }
      }
    })
  })
})

//   __
//  / _|_   _ ________
// | |_| | | |_  /_  /
// |  _| |_| |/ / / /
// |_|  \__,_/___/___|
//

//fuzz tests draw random inputs as fast as the workstation allows for a set number of seconds, so they stay out of the deterministic suite above; set ERGO_FUZZ to a number of seconds to run them, like ERGO_FUZZ=4 pnpm fuzz, which also shows the counts they reached
const fuzzSeconds = Number(process.env.ERGO_FUZZ) || 0

//centers and radii from the tick's original fuzz test: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range
const fuzzCenters = [
  ['2005', 30 * Time.year],
  ['2000feb29', 3 * Time.day],
  ['2000', Time.hour],
  ['2000mar', Time.hour],
  ['2000jan1.12', 6 * Time.hour],
  ['2000jan1.1230', 30 * Time.minute],
  ['2000jan1.1230.30000', 30 * Time.second],
  ['2000', 15 * Time.day],
  ['2000mar', 15 * Time.day],
  ['2000jan1.12', 15 * Time.day],
  ['2000jan1.1230', 12 * Time.hour],
  ['2000jan1.1230.30000', 30 * Time.minute],
  ['1970', Time.day],
  ['9999dec31', Time.day],
]
const lastTick = textToTick('9999dec31.2359.59999') //the ceiling, so a tick drawn near an edge center stays a tick
const everyField = 'Dddd YYYY-MM-DD hh12:mm:ss.ttt AA' //a template that says every field of a tick, the weekday among them

//a random tick within the radius of a center, clamped to the range of ticks
function randomTick(center, radius) {
  let t = center + Math.floor(Math.random() * (2 * radius + 1)) - radius
  return Math.min(Math.max(t, 0), lastTick)
}

//a random span of milliseconds from zero to the integer ceiling, as often small as large, and as often negative
function randomSpan() {
  let size = Math.min(
    Math.floor(10 ** (Math.random() * 16)) - 1,
    Number.MAX_SAFE_INTEGER,
  )
  return Math.random() < 0.5 ? -size : size
}
//the pieces random templates are made of, in groups: the # and its fractions, twice, so most templates hold several; whitespace unicode counts, and three characters it doesn't that a platform's own idea of whitespace might; unit words in each spelling and a mix; and other text, with the long s and the kelvin sign, which a match blind to case would take for s and k
const durationMarks = ['#', '#', '.', '#.#', '#.##', '#.###']
const durationPieces = [
  durationMarks,
  durationMarks,
  [...' \t\n\u00a0\u202f\u3000\u0085\ufeff\u200b\u001c'],
  'y yrs mo Mo MO mos m M min Mins MINUTES w wks'.split(' '),
  'd days Days DAYS h hr hours hOURS s S sec secs seconds'.split(' '),
  ['x', ' and ', ' ago', 'é', '三', '🙂', 'ſ', '\u212a', '$&'],
]
//and for a tick: every tag, the letters and numerals tags are made of, and other text
const tickPieces = [
  'YYYY MMMM Mmmm mmmm MMM Mmm mmm MM M'.split(' '),
  'DDDD Dddd dddd DDD Ddd ddd DD D'.split(' '),
  'hh12 hh24 h12 h24 mm ss ttt tt t AA aa A a'.split(' '),
  'Y m d dd h hh 1 2 4 s T Z'.split(' '),
  [...' :-.,é三🙂', '$&'],
]
//a random template of one to twelve pieces, each from a group picked first, so a short group turns up as often as a long one
function randomTemplate(groups) {
  let pick = (a) => a[Math.floor(Math.random() * a.length)]
  let template = ''
  let length = 1 + Math.floor(Math.random() * 12)
  for (let i = 0; i < length; i++) template += pick(pick(groups))
  return template
}

//a random length, mostly short to land near the 64-byte blocks of the SHA hashes, sometimes long enough to cross several of BLAKE3's 1024-byte chunks
function randomLength() {
  let r = Math.random()
  if (r < 0.5) return Math.floor(Math.random() * 200)
  if (r < 0.8) return Math.floor(Math.random() * 3000)
  return Math.floor(Math.random() * 20000)
}
//data cut into one to four pieces at random places, where a piece is empty when two cuts land together or at an end, and empty data is sometimes no pieces at all
function randomPieces(data) {
  let cuts = Array.from({ length: Math.floor(Math.random() * 4) }, () =>
    Math.floor(Math.random() * (data.size() + 1)),
  ).sort((a, b) => a - b)
  let pieces = []
  let start = 0
  for (let cut of [...cuts, data.size()]) {
    pieces.push(data.clipView(start, cut))
    start = cut
  }
  if (data.size() == 0 && Math.random() < 0.5) return [] //sometimes no pieces at all, which is also the empty input
  return pieces
}

//the characters random text is made of: all 25 of unicode's whitespace characters, the near misses some platform's own trim takes off or that look like whitespace, and a few plain ones, so random text lands on every place the four editions could disagree
const textPieces = [
  ...[0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x20, 0x85, 0xa0, 0x1680],
  ...[0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007],
  ...[0x2008, 0x2009, 0x200a, 0x2028, 0x2029, 0x202f, 0x205f, 0x3000], //unicode's whitespace
  ...[0xfeff, 0x200b, 0x180e, 0x2060, 0x1c, 0x1d, 0x1e, 0x1f], //near misses, none of them on unicode's list
  ...[0x61, 0x62, 0x3d, 0x301, 0x1f600], //a, b, an equals sign, a combining accent, and an emoji outside the basic plane
].map((c) => String.fromCodePoint(c))
//a random text of up to some pieces from textPieces
function randomText(pieces) {
  let s = ''
  for (let i = Math.floor(Math.random() * (pieces + 1)); i > 0; i--)
    s += textPieces[Math.floor(Math.random() * textPieces.length)]
  return s
}

describe.skipIf(!fuzzSeconds)('fuzz', () => {
  test.skipIf(!javascriptChosen)(
    'a random tick round-trips through both text forms in javascript alone, in-process',
    () => {
      let budget = (fuzzSeconds * Time.second) / fuzzCenters.length //milliseconds to spend on each center
      let cycles = 0
      for (let [center, radius] of fuzzCenters) {
        let c = textToTick(center)
        let stop = Date.now() + budget
        while (Date.now() < stop) {
          let t = randomTick(c, radius)
          if (textToTick(tickToText(t)) != t || isoToTick(tickToIso(t)) != t)
            throw new Error(`${t} did not round-trip`) //a plain check rather than expect, which would cost more than the functions under test
          sayTick(t, everyField) //and say it with every field, as the fuzz across the editions does, which is where what it says gets compared
          cycles++
        }
      }
      console.log(
        `javascript alone fuzzed ${cycles} ticks through both text forms and sayTick in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second`,
      )
    },
    (fuzzSeconds + 60) * Time.second,
  )

  test.skipIf(!javascriptChosen)(
    'tags come out as fast as the edition can make them, in javascript alone, in-process',
    () => {
      let stop = Date.now() + fuzzSeconds * Time.second
      let count = 0
      while (Date.now() < stop) {
        let t = Tag()
        if (!hasTag(t)) throw new Error(`${t} is not a tag`)
        count++
      }
      console.log(
        `javascript alone made ${count} tags in ${fuzzSeconds} seconds, ${Math.round(count / fuzzSeconds)} a second, each checked with hasTag`,
      )
    },
    (fuzzSeconds + 60) * Time.second,
  )

  test.skipIf(!javascriptChosen)(
    'each hash runs as fast as the edition can, in javascript alone, in-process',
    async () => {
      let data = Data({ random: Size.mb }) //one mebibyte of random bytes, hashed again and again
      let share = fuzzSeconds / hashPairs.length //an even share of the seconds for each hash
      let said = []
      for (let [name, all, Hasher] of hashPairs) {
        let stop = Date.now() + share * Time.second
        let count = 0
        let last
        do {
          last = await all(data) //web crypto's for sha1 and sha256, noble's for blake3
          count++
        } while (Date.now() < stop) //at least once, however few the seconds
        let h = Hasher()
        h.update(data.clipView(0, 1000)) //and once a piece at a time, at a split that lines up with nothing, which must agree
        h.update(data.clipView(1000, data.size()))
        if (h.digest().base16() != last.base16())
          throw new Error(`${name} disagreed with itself a piece at a time`)
        said.push(`${name} ${Math.round(count / share)} MiB a second`)
      }
      console.log(
        `javascript alone hashed one mebibyte again and again: ${said.join(', ')}`,
      )
    },
    (fuzzSeconds + 60) * Time.second,
  )

  test.skipIf(!javascriptChosen)(
    'visual inspection: 50 random 32-byte values, the size of a sha256 hash, in quoted encoding',
    () => {
      let s = ''
      for (let i = 0; i < 50; i++)
        s += '\n' + dataToQuoted(Data({ random: 32 }))
      console.log(s)
    },
  )

  test.skipIf(!javascriptChosen)(
    'length demonstration: quoted encoding on random 32-byte values, the size of a sha256 hash',
    () => {
      let plain = 0
      let shorter = 0
      let same = 0
      let longer = 0
      let total = 0
      let base16Length = 32 * 2
      let stop = Date.now() + fuzzSeconds * Time.second
      while (Date.now() < stop) {
        let d = Data({ random: 32 })
        let q = dataToQuoted(d)
        let isPlain = q == d.base16()
        if (isPlain) plain++
        else if (q.length < base16Length) shorter++
        else if (q.length > base16Length) longer++
        else same++
        total++
      }
      let quoted = shorter + same + longer
      let percent = (n) => ((n / total) * 100).toFixed(3)
      console.log(
        '\n' +
          deindent`
            ${total} random 32-byte values in ${fuzzSeconds} seconds
            ${percent(plain)}% stayed plain base16
            ${percent(quoted)}% got quoted sections: ${percent(shorter)}% shorter, ${percent(same)}% same, ${percent(longer)}% longer
          `,
      )
    },
    (fuzzSeconds + 60) * Time.second,
  )

  test.skipIf(!javascriptChosen)(
    'quoted encoding round-trips random data, in javascript alone, in-process',
    () => {
      let stop = Date.now() + fuzzSeconds * Time.second
      let cycles = 0
      while (Date.now() < stop) {
        let d = Data({ random: 1 + Math.floor(Math.random() * Size.kb) })
        let q = dataToQuoted(d)
        let u = quotedToData(q)
        if (u.base16() != d.base16())
          throw new Error(`${d.base16()} did not round-trip`) //a plain check rather than expect, which would cost more than the functions under test
        cycles++
      }
      console.log(
        `javascript alone made ${cycles} quoted encoding round trips in ${fuzzSeconds} seconds`,
      )
    },
    (fuzzSeconds + 60) * Time.second,
  )

  describe('every edition', () => {
    let editions
    beforeAll(async () => {
      editions = await startEditions()
    }, 180_000)
    afterAll(() => stopEditions(editions))

    test(
      'a random tick round-trips through both text forms in every edition, and every edition agrees',
      async () => {
        let budget = (fuzzSeconds * Time.second) / fuzzCenters.length
        let cycles = 0
        for (let [center, radius] of fuzzCenters) {
          let c = textToTick(center)
          let stop = Date.now() + budget
          while (Date.now() < stop) {
            let t = randomTick(c, radius)
            let text = tickToText(t)
            let iso = tickToIso(t)
            let said = sayTick(t, everyField)
            await Promise.all(
              Object.entries(editions).map(async ([name, e]) => {
                expect(await e.tickToText(t), name).toBe(text) //every edition writes the same text, which pins rust's hand-rolled calendar against three platforms
                expect(await e.tickToIso(t), name).toBe(iso)
                expect(await e.sayTick(t, everyField), name).toBe(said) //and says it the same way, which pins rust's weekday arithmetic too
                expect(await e.textToTick(text), name).toBe(t) //and reads it back exactly
                expect(await e.isoToTick(iso), name).toBe(t)
              }),
            )
            cycles++
          }
        }
        console.log(
          `every edition fuzzed ${cycles} ticks through both text forms and sayTick in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second, each a round trip over the pipes to three drivers`,
        )
      },
      (fuzzSeconds + 60) * Time.second,
    )

    test(
      "a random template and span say the same in every edition, which pins rust's two scanners against three platforms' regular expressions",
      async () => {
        let stop = Date.now() + fuzzSeconds * Time.second
        let cycles = 0
        while (Date.now() < stop) {
          let span = randomSpan()
          let duration = randomTemplate(durationPieces)
          let t = Math.floor(Math.random() * (lastTick + 1)) //any tick in the range, since what's random here is the template
          let tick = randomTemplate(tickPieces)
          let said = [
            sayDuration(span, duration),
            sayTick(t, tick),
            sayTimeRemaining(span), //and the shortcuts on the same span
            sayTimeRemainingCoarse(span),
            sayTimeRace(span),
          ]
          await Promise.all(
            Object.entries(editions).map(async ([name, e]) => {
              expect(
                [
                  await e.sayDuration(span, duration),
                  await e.sayTick(t, tick),
                  await e.sayTimeRemaining(span),
                  await e.sayTimeRemainingCoarse(span),
                  await e.sayTimeRace(span),
                ],
                `${name}: ${JSON.stringify([span, duration, t, tick])}`, //the inputs, so a mismatch can be pinned as a vector
              ).toEqual(said)
            }),
          )
          cycles++
        }
        console.log(
          `every edition fuzzed ${cycles} random templates through sayDuration and sayTick, and spans through the shortcuts, in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second`,
        )
      },
      (fuzzSeconds + 60) * Time.second,
    )

    test(
      'random bytes hash the same in every edition, all at once and a piece at a time',
      async () => {
        let stop = Date.now() + fuzzSeconds * Time.second
        let cycles = 0
        while (Date.now() < stop) {
          let data = Data({ random: randomLength() })
          let whole = { base16: data.base16() } //bytes cross to the drivers tagged, as the vectors carry them
          let pieces = randomPieces(data).map((p) => ({ base16: p.base16() }))
          await Promise.all(
            Object.entries(editions).map(async ([name, e]) => {
              for (let [hash, all, Hasher] of hashPairs) {
                let expected = { base16: (await all(data)).base16() } //javascript's answer in-process, which every edition's must match
                expect(await e[hash](whole), name).toEqual(expected)
                expect(await e[Hasher.name](pieces), name).toEqual(expected) //and the same a piece at a time, from the hasher by its protocol name, like Sha1; this is what pins noble's blake3 against the official implementation in rust and python
              }
            }),
          )
          cycles++
        }
        console.log(
          `every edition fuzzed ${cycles} random inputs through all three hashes, both forms, in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second`,
        )
      },
      (fuzzSeconds + 60) * Time.second,
    )

    test(
      'random text with every kind of whitespace trims, asks, cuts, and deindents the same in every edition',
      async () => {
        let stop = Date.now() + fuzzSeconds * Time.second
        let cycles = 0
        while (Date.now() < stop) {
          let s = randomText(12)
          let tag = randomText(2) || 'a' //a tag of at least one character, which may or may not be in the text
          let expected = {
            trim: trim(s),
            hasText: hasText(s),
            cut: cut(s, tag),
            cutLast: cutLast(s, tag),
            deindent: deindent([s]),
          } //javascript's answers in-process, which every edition's must match; deindent takes the text as a template's one piece
          for (let [name, e] of Object.entries(editions)) {
            let answer = {
              trim: await e.trim(s),
              hasText: await e.hasText(s),
              cut: await e.cut(s, tag),
              cutLast: await e.cutLast(s, tag),
              deindent: await e.deindent(s),
            }
            expect(
              answer,
              `${name} given ${JSON.stringify(s)} and the tag ${JSON.stringify(tag)}`,
            ).toEqual(expected) //compared as values, since an edition's json can write an object's keys in another order
          }
          cycles++
        }
        console.log(
          `every edition fuzzed ${cycles} random texts through trim, hasText, cut, cutLast, and deindent in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second`,
        )
      },
      (fuzzSeconds + 60) * Time.second,
    )

    test(
      'random tiny files with random tiny protocols get the same fuji hashes in every edition, from a file, from a stream, and fed to a hasher',
      async () => {
        let stop = Date.now() + fuzzSeconds * Time.second
        let cycles = 0
        while (Date.now() < stop) {
          let between = (low, high) =>
            low + Math.floor(Math.random() * (high - low + 1))
          let protocol = {
            pieces: {
              title: 'Test.Pieces.SHA256.10-20B.',
              size: between(10, 20),
            },
            tips: { title: 'Test.Tips.SHA256.5-15B.', size: between(5, 15) },
          } //small files and tiny block sizes, the way the original, the JavaScript ergo grew from, fuzzes them, so every case crosses many pieces and stripes
          let data = Data({ random: between(1, 500) })
          let file = { base16: data.base16() }
          let pieces = randomPieces(data).map((p) => ({ base16: p.base16() }))
          let fed = randomPieces(data).map((p) => ({ base16: p.base16() })) //cut another way, for the hasher
          let expected = await call('hashStream', [pieces, protocol])
          let tip = await call('hashFile', [file, protocol])
          expect(tip.tipHash).toEqual(expected.tipHash) //importantly, make sure we get the same tip hash from slicing the file and peeking at the stream!
          await Promise.all(
            Object.entries(editions).map(async ([name, e]) => {
              expect(await e.hashStream(pieces, protocol), name).toEqual(
                expected,
              )
              expect(await e.hashFile(file, protocol), name).toEqual(tip)
              expect(
                await e.FujiHasher(data.size(), fed, protocol),
                name,
              ).toEqual(expected)
            }),
          )
          cycles++
        }
        console.log(
          `every edition fuzzed ${cycles} tiny files through both fuji hashes, from a file, a stream, and a hasher, in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second`,
        )
      },
      (fuzzSeconds + 60) * Time.second,
    )

    test(
      'random realistic files, with the real protocol, get the same fuji hashes in every edition, from a file, from a stream, and fed to a hasher',
      async () => {
        let stop = Date.now() + fuzzSeconds * Time.second
        let cycles = 0
        while (Date.now() < stop) {
          let size = 1 + Math.floor(Math.random() * 20 * Size.mb) //realistic files, as the original tests them, up to 20 MiB, across several 4 MiB pieces
          let unit = Data({ random: 1 + Math.floor(Math.random() * 5000) })
          let file = { repeat: unit.base16(), length: size } //random bytes repeated to the size, so each edition makes the file from a line of json rather than 40 MB of base16
          let a = new Uint8Array(size)
          for (let i = 0; i < size; i += unit.size())
            a.set(unit.array().subarray(0, size - i), i) //the same bytes here, in-process
          let expected = await call('hashStream', [[file]])
          let tip = await call('hashFile', [file])
          expect(tip.tipHash).toEqual(expected.tipHash) //importantly, make sure we get the same tip hash from slicing the file and peeking at the stream!
          let hasher = FujiHasher({ size })
          for (let piece of randomPieces(Data({ array: a })))
            await hasher.update(piece) //pieces of any size, some larger than the conveyor belt
          let both = await hasher.digest()
          expect(both.tipHash.base16()).toBe(expected.tipHash.base16)
          expect(both.pieceHash.base16()).toBe(expected.pieceHash.base16)
          await Promise.all(
            Object.entries(editions).map(async ([name, e]) => {
              expect(await e.hashStream([file]), name).toEqual(expected)
              expect(await e.hashFile(file), name).toEqual(tip)
              expect(await e.FujiHasher(size, [file]), name).toEqual(expected)
            }),
          )
          cycles++
        }
        console.log(
          `every edition fuzzed ${cycles} realistic files through both fuji hashes, from a file, a stream, and a hasher, in ${fuzzSeconds} seconds`,
        )
      },
      (fuzzSeconds + 120) * Time.second,
    )

    test(
      'the original realistic files, with the real protocol, hash to the original answers in every edition',
      async () => {
        let b16 = (s) => ({ base16: Data({ base32: s }).base16() })
        for (let [tip, piece, seed, size] of [
          [
            '3UHG26C6MLS3DDNBQIEPWKI23P7TBXTLW36ENWYOFTDSLMOS2MRA',
            'I464HASPP45J62P4IC3ANMDWMJKWGLSKBTJQGYLD4PQ5KQIPFEJQ',
            6317,
            5911493,
          ],
          [
            'FI5OSG7AGTDWIC7EGIOCT4RHUZ3ML2K4JALXH4F2SNMHSQKA666Q',
            '63TIRHGUKSBOI54P6BPFW56GVCOIKXGWVQEDY353P6S4MX7YZIMA',
            927,
            6427647,
          ],
        ]) {
          let file = { mulberry: seed, length: size } //medium files 🖼️, more than one 4 MiB piece each
          for (let [name, e] of Object.entries(editions)) {
            expect(await e.hashFile(file), name).toEqual({ tipHash: b16(tip) })
            expect(await e.hashStream([file]), name).toEqual({
              tipHash: b16(tip),
              pieceHash: b16(piece),
            })
            expect(await e.FujiHasher(size, [file]), name).toEqual({
              tipHash: b16(tip),
              pieceHash: b16(piece),
            }) //and all of it in one update, more than a piece at once
          }
        }
        //the original 19.7 MB and 28 MB files can't be pinned this way: their mulberry seed grows past the integers a double holds exactly at about 19.7 MB, so no generator that wraps at 32 bits can make their last bytes
      },
      (fuzzSeconds + 120) * Time.second,
    )
  })
})

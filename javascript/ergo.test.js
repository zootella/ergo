import { test, expect, describe, beforeAll, afterAll } from 'vitest'
import {
  Size,
  saySize4,
  Time,
  tickToText,
  textToTick,
  tickToIso,
  isoToTick,
  hasTick,
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
} from './ergo.js'
import { call } from './driver.js'
import { startEditions, stopEditions } from './editions.js'
import { readdirSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

//                 _
// __   _____  ___| |_ ___  _ __ ___
// \ \ / / _ \/ __| __/ _ \| '__/ __|
//  \ V /  __/ (__| || (_) | |  \__ \
//   \_/ \___|\___|\__\___/|_|  |___/
//

//the shared vectors beside this edition: every case from every json file in tests/
const vectorsDirectory = join(
  dirname(fileURLToPath(import.meta.url)),
  '..',
  'tests',
)
const vectors = readdirSync(vectorsDirectory)
  .filter((f) => f.endsWith('.json'))
  .flatMap((f) => JSON.parse(readFileSync(join(vectorsDirectory, f), 'utf8')))

//name a case the way it reads: saySize4(10000) five digits of bytes, so the next unit up
function describeVector(c) {
  return `${c.function}(${c.input.map((a) => JSON.stringify(a)).join(', ')}) ${c.note ?? ''}`
}

describe('vectors, javascript in-process', () => {
  for (let c of vectors) {
    test(describeVector(c), async () => {
      if (c.throws) await expect(call(c.function, c.input)).rejects.toThrow()
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

describe('Size', () => {
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

describe('Time', () => {
  test('names a span of milliseconds', () => {
    // #region time
    let deadline = 1700000000000 + 2 * Time.hour //two hours after a tick, which counts milliseconds since 1970
    // #endregion time
    expect(deadline).toBe(1700007200000)
  })
})

//  _   _      _
// | |_(_) ___| | __
// | __| |/ __| |/ /
// | |_| | (__|   <
//  \__|_|\___|_|\_\
//

describe('tick', () => {
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
})

//  _
// | |_ __ _  __ _
// | __/ _` |/ _` |
// | || (_| | (_| |
//  \__\__,_|\__, |
//           |___/

describe('Tag', () => {
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

//  _               _
// | |__   __ _ ___| |__
// | '_ \ / _` / __| '_ \
// | | | | (_| \__ \ | | |
// |_| |_|\__,_|___/_| |_|
//

//lowercase base16, the form the world shows a hash in; a test can lean on node's Buffer, where the edition itself can't
function hex(bytes) {
  return Buffer.from(bytes).toString('hex')
}

//the three hashes by name, with the function that takes all the bytes at once and the hasher that takes a piece at a time
const hashPairs = [
  ['sha1', sha1, Sha1],
  ['sha256', sha256, Sha256],
  ['blake3', blake3, Blake3],
]

describe('hash', () => {
  test('hashes bytes all at once, and a file a piece at a time', async () => {
    // #region hash
    let d = sha256(new TextEncoder().encode('hello')) //all at once, for bytes already in memory: 32 bytes
    let file = new Blob(['hello']) //or a File the user chose, of any size
    let hashers = [Sha1(), Sha256(), Blake3()] //or a piece at a time, all three from one read of the file
    let reader = file.stream().getReader()
    while (true) {
      let { done, value } = await reader.read()
      if (done) break
      for (let h of hashers) h.update(value)
    }
    let [d1, d256, db3] = hashers.map((h) => h.digest())
    // #endregion hash
    expect(hex(d)).toBe(
      '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824',
    )
    expect(hex(d1)).toBe('aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d')
    expect(hex(d256)).toBe(hex(d))
    expect(hex(db3)).toBe(
      'ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f',
    )
  })
  test("digest doesn't finish a hasher, which keeps going for the hash of everything so far", () => {
    let a = new Uint8Array([0x61])
    let ab = new Uint8Array([0x61, 0x62])
    for (let [name, all, Hasher] of hashPairs) {
      let h = Hasher()
      h.update(a)
      expect(hex(h.digest()), name).toBe(hex(all(a)))
      expect(hex(h.digest()), name).toBe(hex(all(a))) //a second digest is the same answer
      h.update(ab.subarray(1))
      expect(hex(h.digest()), name).toBe(hex(all(ab))) //and the hasher took more after both
    }
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
          if (c.throws)
            await expect(e[c.function](...c.input), name).rejects.toThrow()
          else expect(await e[c.function](...c.input), name).toEqual(c.output)
        }
      })
    }
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

//centers and radii from cold3: a broad sweep of decades, leap day, each unit boundary at a narrow and then a wide aperture, and the two edges of the range
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

//a random tick within the radius of a center, clamped to the range of ticks
function randomTick(center, radius) {
  let t = center + Math.floor(Math.random() * (2 * radius + 1)) - radius
  return Math.min(Math.max(t, 0), lastTick)
}

//n random bytes; getRandomValues fills at most 65536 at a call, so fill in steps
function randomBytes(n) {
  let b = new Uint8Array(n)
  for (let i = 0; i < n; i += 65536)
    crypto.getRandomValues(b.subarray(i, Math.min(i + 65536, n)))
  return b
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
    Math.floor(Math.random() * (data.length + 1)),
  ).sort((a, b) => a - b)
  let pieces = []
  let start = 0
  for (let cut of [...cuts, data.length]) {
    pieces.push(data.subarray(start, cut))
    start = cut
  }
  if (data.length == 0 && Math.random() < 0.5) return [] //sometimes no pieces at all, which is also the empty input
  return pieces
}

describe.skipIf(!fuzzSeconds)('fuzz', () => {
  test(
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
          cycles++
        }
      }
      console.log(
        `javascript alone fuzzed ${cycles} ticks through both text forms in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second`,
      )
    },
    (fuzzSeconds + 60) * Time.second,
  )

  test(
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

  test(
    'each hash runs as fast as the edition can, in javascript alone, in-process',
    () => {
      let data = randomBytes(Size.mb) //one mebibyte of random bytes, hashed again and again
      let share = fuzzSeconds / hashPairs.length //an even share of the seconds for each hash
      let said = []
      for (let [name, all, Hasher] of hashPairs) {
        let stop = Date.now() + share * Time.second
        let count = 0
        let last
        do {
          last = all(data)
          count++
        } while (Date.now() < stop) //at least once, however few the seconds
        let h = Hasher()
        h.update(data.subarray(0, 1000)) //and once a piece at a time, at a split that lines up with nothing, which must agree
        h.update(data.subarray(1000))
        if (hex(h.digest()) != hex(last))
          throw new Error(`${name} disagreed with itself a piece at a time`)
        said.push(`${name} ${Math.round(count / share)} MiB a second`)
      }
      console.log(
        `javascript alone hashed one mebibyte again and again: ${said.join(', ')}`,
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
            await Promise.all(
              Object.entries(editions).map(async ([name, e]) => {
                expect(await e.tickToText(t), name).toBe(text) //every edition writes the same text, which pins rust's hand-rolled calendar against three platforms
                expect(await e.tickToIso(t), name).toBe(iso)
                expect(await e.textToTick(text), name).toBe(t) //and reads it back exactly
                expect(await e.isoToTick(iso), name).toBe(t)
              }),
            )
            cycles++
          }
        }
        console.log(
          `every edition fuzzed ${cycles} ticks through both text forms in ${fuzzSeconds} seconds, ${Math.round(cycles / fuzzSeconds)} a second, each a round trip over the pipes to three drivers`,
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
          let data = randomBytes(randomLength())
          let whole = { base16: hex(data) } //bytes cross to the drivers tagged, as the vectors carry them
          let pieces = randomPieces(data).map((p) => ({ base16: hex(p) }))
          await Promise.all(
            Object.entries(editions).map(async ([name, e]) => {
              for (let [hash, all, Hasher] of hashPairs) {
                let expected = { base16: hex(all(data)) } //javascript's answer in-process, which every edition's must match
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
  })
})

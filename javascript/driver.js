//the javascript edition's driver: one json request per line on stdin, one json response per line on stdout
//the vitest suite talks to this edition in-process and uses drivers for the other three, but every edition ships one, so any harness can drive any edition
import * as ergo from './ergo.js'
import { createInterface } from 'node:readline'

//run the named function with json-decoded arguments; a constant is a function with no arguments, and a dotted name like "Size.kb" reaches into a namespace of them
export async function call(name, args) {
  args = args.map(fromProtocol)
  if (Object.hasOwn(adapters, name))
    return toProtocol(await adapters[name](...args))
  let f = ergo
  for (let part of name.split('.')) f = f?.[part]
  if (f == undefined) throw new Error(`unknown function: ${name}`)
  return toProtocol(typeof f == 'function' ? await f(...args) : f)
}

//the protocol's names for things that keep state, each answering in one call, so a vector pins them without the harness keeping an object between calls
const adapters = {
  //a hasher takes one argument, a list of pieces, and answers the hash of all of them fed in order
  Sha1: (pieces) => fed(ergo.Sha1(), pieces),
  Sha256: (pieces) => fed(ergo.Sha256(), pieces),
  Blake3: (pieces) => fed(ergo.Blake3(), pieces),
  //a Data takes the form to make it from, then the name of a method and that method's arguments, like [{text: 'ABC'}, 'base16'] or [{base16: '414243'}, 'clipView', 1, 3]
  Data: (form, method, ...args) => {
    let d = form?.type == 'Data' ? form : ergo.Data(form) //tagged bytes arrive already a Data; any other form, like {text}, makes one here
    return d[method](...args)
  },
  //a Bin takes its capacity, a list of what to add, each a byte or a Data, and the name of a method to answer with
  Bin: (capacity, items, method) => {
    let b = ergo.Bin(capacity)
    for (let item of items) b.add(item)
    return b[method]()
  },
  //hashMeasure answers what every edition measures alike: the pieces, and the stripes the tip hash reads
  hashMeasure: (measure) => {
    let o = ergo.hashMeasure(measure)
    return {
      pieces: o.pieces,
      complete: o.complete,
      fragment: o.fragment,
      stripes: o.stripes,
      stripeSize: o.stripeSize,
    }
  },
  //hashFile takes the file's bytes as a Data, and a protocol, or none for the fuji one, and answers the tip hash
  hashFile: async (file, protocol) => {
    let status = await ergo.hashFile({
      file: new Blob([file.array()]),
      size: file.size(),
      protocol,
    })
    return { tipHash: status.tipHash }
  },
  //hashStream takes the stream as a list of pieces, which arrive as its boxes in order, and a protocol, and answers both hashes
  hashStream: async (pieces, protocol) => {
    let size = pieces.reduce((total, piece) => total + piece.size(), 0)
    let stream = new ReadableStream({
      start(controller) {
        for (let piece of pieces) controller.enqueue(piece.array())
        controller.close()
      },
    })
    let status = await ergo.hashStream({ stream, size, protocol })
    return { tipHash: status.tipHash, pieceHash: status.pieceHash }
  },
}
function fed(h, pieces) {
  for (let piece of pieces) h.update(piece)
  return h.digest()
}

//bytes cross the protocol as tagged json, which becomes a Data: {base16: '616263'} is those three bytes, and {repeat: '61', length: 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line; anything else passes through as it is
function fromProtocol(v) {
  if (Array.isArray(v)) return v.map(fromProtocol)
  if (typeof v != 'object' || v === null) return v //text, numbers, and booleans pass as they are; ask this first, since the in operator below works only on objects
  if ('base16' in v) return ergo.Data({ base16: v.base16 }) //ergo's own base16 reader, which takes exactly what its writer makes, lowercase and nothing else
  if ('mulberry' in v) return mulberryData(v.mulberry, v.length)
  if ('repeat' in v) {
    let unit = ergo.Data({ base16: v.repeat }).array()
    if (unit.length == 0 || !Number.isSafeInteger(v.length) || v.length < 0)
      throw new Error(`not bytes: ${JSON.stringify(v)}`) //a unit of at least one byte, repeated to a count within the integer ceiling, the same rule in every edition
    let a = new Uint8Array(v.length)
    for (let i = 0; i < a.length; i++) a[i] = unit[i % unit.length]
    return ergo.Data({ array: a })
  }
  return v
}
//and a result that is a Data goes back the first way, as its base16, wherever it is in the answer
function toProtocol(v) {
  if (v?.type == 'Data') return { base16: v.base16() }
  if (Array.isArray(v)) return v.map(toProtocol)
  if (v?.constructor == Object)
    return Object.fromEntries(
      Object.entries(v).map(([k, x]) => [k, toProtocol(x)]),
    )
  return v
}

//{mulberry: seed, length: n} is n bytes from icarus's Mulberry32 generator at that seed, random-looking but the same every time, so a vector can name a realistic file of any size in a line
//Mulberry32 is a well-known small PRNG by Tommy Ettinger from 2018; this one wraps at 32 bits, as the other editions must, which matches icarus's for any file under about 19 MB, past which icarus's seed outgrows a double's exact integers
function mulberryData(seed, length) {
  let s = seed >>> 0
  function rng() {
    s = (s + 0x6d2b79f5) >>> 0
    let t = s
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return (t ^ (t >>> 14)) >>> 0
  }
  let a = new Uint8Array(length)
  let v = 0
  for (let i = 0; i < length; i++) {
    if (i % 4 == 0) v = rng() //a new 32-bit value every 4 bytes
    a[i] = (v >>> ((i % 4) * 8)) & 0xff //the appropriate byte from it, low byte first
  }
  return ergo.Data({ array: a })
}

if (process.argv[1] == new URL(import.meta.url).pathname) {
  //started as a program rather than imported: serve requests until stdin closes
  for await (let line of createInterface({ input: process.stdin })) {
    let request = JSON.parse(line)
    let response = { id: request.id }
    try {
      response.result = await call(request.call, request.args)
    } catch (e) {
      response.error = e.message //the top gate: report anything that escaped as the response, and keep serving
    }
    process.stdout.write(JSON.stringify(response) + '\n')
  }
}

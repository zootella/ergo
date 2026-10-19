//the javascript edition's driver: one json request per line on stdin, one json response per line on stdout
//the vitest suite talks to this edition in-process and uses drivers for the other three, but every edition ships one, so any harness can drive any edition
import * as ergo from './ergo.js'
import { createInterface } from 'node:readline'

//run the named function with json-decoded arguments; a constant is a function with no arguments, and a dotted name like "Size.kb" reaches into a namespace of them
export async function call(name, args) {
  args = args.map(fromProtocol)
  if (Object.hasOwn(hashers, name))
    return toProtocol(fed(hashers[name](), args[0]))
  let f = ergo
  for (let part of name.split('.')) f = f?.[part]
  if (f == undefined) throw new Error(`unknown function: ${name}`)
  return toProtocol(typeof f == 'function' ? await f(...args) : f)
}

//a hasher in the protocol takes one argument, a list of pieces, and answers the hash of all of them fed in order, so a vector pins the piece-at-a-time form without the harness keeping an object between calls
const hashers = { Sha1: ergo.Sha1, Sha256: ergo.Sha256, Blake3: ergo.Blake3 }
function fed(h, pieces) {
  for (let piece of pieces) h.update(piece)
  return h.digest()
}

//bytes cross the protocol as tagged json: {base16: '616263'} is those three bytes, and {repeat: '61', length: 1000000} is those bytes repeated to that length, so a vector can name a million bytes in a line; anything else passes through as it is
function fromProtocol(v) {
  if (Array.isArray(v)) return v.map(fromProtocol)
  if (typeof v != 'object' || v === null) return v //text, numbers, and booleans pass as they are; ask this first, since the in operator below works only on objects
  if ('base16' in v) return base16ToBytes(v.base16)
  if ('repeat' in v) {
    let unit = base16ToBytes(v.repeat)
    if (unit.length == 0 || !Number.isSafeInteger(v.length) || v.length < 0)
      throw new Error(`not bytes: ${JSON.stringify(v)}`) //a unit of at least one byte, repeated to a count within the integer ceiling, the same rule in every edition
    let b = new Uint8Array(v.length)
    for (let i = 0; i < b.length; i++) b[i] = unit[i % unit.length]
    return b
  }
  return v
}
//and a result that is bytes goes back the first way, in lowercase base16
function toProtocol(v) {
  return v instanceof Uint8Array
    ? { base16: Buffer.from(v).toString('hex') }
    : v
}
//bytes from base16, strictly: an even number of the digits 0 through 9 and a through f, in either case, and nothing else, where node's decoder would stop quietly at the first character it can't read
function base16ToBytes(s) {
  if (typeof s != 'string' || !/^([0-9a-f]{2})*$/i.test(s))
    throw new Error(`not base16: ${JSON.stringify(s)}`)
  return new Uint8Array(Buffer.from(s, 'hex')) //a plain Uint8Array rather than node's Buffer, so the edition sees what a page would hand it
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

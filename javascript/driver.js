//the javascript edition's driver: one json request per line on stdin, one json response per line on stdout
//the vitest suite talks to this edition in-process and uses drivers for the other three, but every edition ships one, so any harness can drive any edition
import * as ergo from './ergo.js'
import { createInterface } from 'node:readline'

//run the named function with json-decoded arguments; a constant is a function with no arguments, and a dotted name like "Size.kb" reaches into a namespace of them
export async function call(name, args) {
  let f = ergo
  for (let part of name.split('.')) f = f?.[part]
  if (f == undefined) throw new Error(`unknown function: ${name}`)
  return typeof f == 'function' ? await f(...args) : f
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

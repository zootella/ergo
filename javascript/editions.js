//start the four editions of ergo for a test to drive side by side: javascript in-process, and python, rust, and go as driver processes speaking json lines over stdin and stdout
import { call } from './driver.js'
import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const root = join(dirname(fileURLToPath(import.meta.url)), '..') //the repository root, which holds the four edition directories

//how to launch each foreign driver; each command compiles on first use, and the toolchain caches it after that
const commands = {
  python: {
    command: 'uv',
    args: ['run', '--quiet', 'python', '-m', 'zootella.ergo.driver'], //in the edition's own environment, which has its dependency, blake3, and the edition itself installed
    cwd: 'python',
  },
  rust: {
    command: 'cargo',
    args: ['run', '--quiet', '--features', 'driver', '--bin', 'driver'],
    cwd: 'rust',
  },
  go: { command: 'go', args: ['run', './cmd/driver'], cwd: 'go' },
}

const names = ['javascript', 'python', 'rust', 'go'] //every edition, in the order the suite drives them

//the editions a run drives: all four, or just the ones ERGO_EDITIONS names, like ERGO_EDITIONS=javascript,python, so a family's vectors can be proven against one edition before the others are written
//a name that isn't an edition throws, so a typo can't quietly turn an edition off
export function chosenEditions() {
  let s = process.env.ERGO_EDITIONS
  if (!s) return names
  let chosen = s.split(',').map((name) => name.trim())
  for (let name of chosen)
    if (!names.includes(name))
      throw new Error(
        `ERGO_EDITIONS names ${JSON.stringify(name)}, which isn't one of ${names.join(', ')}`,
      )
  return names.filter((name) => chosen.includes(name)) //in the usual order, however they were written
}

//start the chosen editions and return them, like {javascript, python, rust, go}; each is an object whose functions are ergo's, returning promises
export async function startEditions(chosen = chosenEditions()) {
  let editions = {}
  if (chosen.includes('javascript'))
    editions.javascript = edition(
      (name, args) => call(name, args),
      () => {},
    )
  for (let [name, c] of Object.entries(commands))
    if (chosen.includes(name)) editions[name] = await startDriver(name, c)
  return editions
}
//stop the driver processes
export function stopEditions(editions) {
  for (let e of Object.values(editions)) e.stop()
}

//wrap a call function as an edition: e.saySize4(n) becomes send('saySize4', [n]), with stop kept aside from the function names
function edition(send, stop) {
  return new Proxy(
    {},
    {
      get(_, name) {
        if (name == 'stop') return stop
        if (name == 'then') return undefined //not a promise, which await would otherwise check for
        return (...args) => send(name, args)
      },
    },
  )
}

//spawn a foreign driver and return it as an edition
async function startDriver(name, c) {
  let child = spawn(c.command, c.args, {
    cwd: join(root, c.cwd),
    env: { ...process.env, ...c.env },
    stdio: ['pipe', 'pipe', 'inherit'], //we write requests, read responses, and let the toolchain's errors through to the console
  })
  let pending = new Map() //request id to {resolve, reject} for calls awaiting a response
  let next = 1

  createInterface({ input: child.stdout }).on('line', (line) => {
    let response = JSON.parse(line)
    let p = pending.get(response.id)
    pending.delete(response.id)
    if ('error' in response) p.reject(new Error(`${name}: ${response.error}`))
    else p.resolve(response.result)
  })
  let fail = (reason) => {
    for (let p of pending.values())
      p.reject(new Error(`${name} driver: ${reason}`))
    pending.clear()
  }
  child.on('exit', (code) => fail(`exited with code ${code}`))
  child.on('error', (e) => fail(e.message))

  let send = (fn, args) =>
    new Promise((resolve, reject) => {
      let id = next++
      pending.set(id, { resolve, reject })
      child.stdin.write(JSON.stringify({ id, call: fn, args }) + '\n')
    })
  let e = edition(send, () => child.kill())
  await e['Size.b']() //wait for the process to come up, which includes compiling the first time, and confirm it answers; a constant is the simplest call
  return e
}

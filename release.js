/*
Release ergo: one command takes all four editions to their registries at a new version.

  pnpm release patch     0.1.0 becomes 0.1.1, for a release that adds
  pnpm release minor     0.1.0 becomes 0.2.0, for one that changes an answer ergo already gave
  pnpm release 0.3.0     or name the version exactly

The work is split between this machine and GitHub. Here, the command checks that main is clean and up to date, pushes it if GitHub hasn't seen it, and waits for GitHub's tests to pass on it. Then it sets the version in the five files that hold it, commits that as "ergo 0.1.1", tags the commit v0.1.1, and pushes both. On GitHub, the tag starts .github/workflows/publish.yml, which tests the tagged commit again, publishes to npm, PyPI, and crates.io by trusted publishing, tags go/v0.1.1 for the Go module, and makes a GitHub release that links all four. The command watches that run and ends with the release's address, so this terminal says whether it worked, and no registry's site has to be checked by hand.

That split is the ordinary one, npm version and then git push --follow-tags, widened to four ecosystems. The version has to live in the manifests, since each registry reads it from there, so a commit made here carries it. Publishing needs the identity each registry was told to trust, which only the workflow has, so GitHub does it. A failure after the tag is pushed loses nothing: every step of the workflow skips what's already done, so running it again finishes the release.

The repository root belongs to no edition, and its package.json is private and holds nothing but the release script, so pnpm release at the root runs node release.js; pnpm's own patch command is why the verb is release rather than patch. This file imports only node's own modules, and nothing needs installing at the root. It runs git, gh, which has to be logged in, and npm, uv, and cargo, so each ecosystem writes its own version its own way.
*/
import { execFileSync, spawnSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { setTimeout as sleep } from 'node:timers/promises'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('.', import.meta.url)) //the repository root, where every path and command here starts, wherever node was started

//the five files that hold the version, which are all the release commit changes
const versionFiles = [
  'javascript/package.json',
  'python/pyproject.toml',
  'python/uv.lock', //uv records the project's own version beside its dependencies
  'rust/Cargo.toml',
  'rust/Cargo.lock', //and so does cargo
]

async function main() {
  let current = readVersion()
  let version = nextVersion(current, process.argv[2])
  let tag = `v${version}`
  console.log(`ergo ${current} to ${version}`)

  //check everything that could stop the release before changing anything
  capture('gh auth status') //says how to log in, if gh isn't
  if (capture('git branch --show-current') != 'main')
    throw new Error('release from main')
  if (capture('git status --porcelain --untracked-files=no'))
    throw new Error('main has changes that are not committed')
  run('git fetch --quiet origin main')
  if (capture('git rev-list --count HEAD..origin/main') != '0')
    throw new Error('origin has commits this main lacks; pull first')
  if (
    capture(`git tag --list ${tag} go/${tag}`) ||
    capture(`git ls-remote --tags origin ${tag}`) //on GitHub too, where a pattern matches the end of a name, so it finds go/v0.1.1 as well as v0.1.1
  )
    throw new Error(`${tag} is already tagged`)

  //release only what GitHub has tested, pushing main first if it holds commits GitHub hasn't seen
  if (capture('git rev-list --count origin/main..HEAD') != '0')
    run('git push --quiet origin main')
  let tested = capture('git rev-parse HEAD')
  console.log(`waiting for the tests on ${tested.slice(0, 7)}`)
  await watchRun('test.yml', tested, 'the tests failed; nothing was released')

  //set the version, and confirm the five files changed and nothing else did
  writeVersion(version)
  let changed = capture('git diff --name-only').split('\n')
  if (changed.sort().join() != [...versionFiles].sort().join())
    throw new Error(`setting the version changed ${changed.join(', ')}`)
  if (readVersion() != version)
    throw new Error(`the five files don't all say ${version}`)

  //commit, tag, and push the two at once, so GitHub never has one without the other
  run('git commit --quiet --message', `ergo ${version}`, '--', ...versionFiles)
  run(`git tag ${tag}`)
  run(`git push --quiet --atomic origin main ${tag}`)

  console.log(`publishing ${tag}`)
  await watchRun(
    'publish.yml',
    capture('git rev-parse HEAD'),
    'publishing failed; once the trouble is fixed, rerun the failed jobs on this page, and each step will skip what it already did',
  )
  console.log(`released https://github.com/zootella/ergo/releases/tag/${tag}`)
  console.log('npm can take a few minutes to list a new version') //npm processes each one before serving it, which took six minutes for 0.1.2
}

//the version all five files hold; a release starts from their agreement, since the last one left them so, and checks it again after writing the next
function readVersion() {
  let text = (path) => readFileSync(join(root, path), 'utf8')
  let manifest = /^version = "(.+)"$/m //the first version line in a toml manifest is the package's own, since dependencies give theirs inline
  let lock = /name = "zootella-ergo"\nversion = "(.+)"/ //ergo's own entry in a lockfile
  let versions = [
    JSON.parse(text('javascript/package.json')).version,
    text('python/pyproject.toml').match(manifest)[1],
    text('python/uv.lock').match(lock)[1],
    text('rust/Cargo.toml').match(manifest)[1],
    text('rust/Cargo.lock').match(lock)[1],
  ]
  if (new Set(versions).size != 1)
    throw new Error(`the five files disagree: ${versions.join(', ')}`)
  return versions[0]
}

//the version to release, from the command's one argument: patch and minor count up from the current version, and an exact version has to be above it
function nextVersion(current, argument) {
  let [major, minor, patch] = current.split('.').map(Number)
  if (argument == 'patch') return `${major}.${minor}.${patch + 1}`
  if (argument == 'minor') return `${major}.${minor + 1}.0`
  if (!/^\d+\.\d+\.\d+$/.test(argument))
    throw new Error('say patch, minor, or a version like 0.3.0')
  let [a, b] = [argument, current].map((v) => v.split('.').map(Number))
  let i = a.findIndex((n, j) => n != b[j]) //the first of major, minor, and patch where the two differ, which decides which is higher
  if (i == -1 || a[i] < b[i])
    throw new Error(`${argument} is not above ${current}`)
  return argument
}

//write the version the way each ecosystem writes it, from the root and offline
function writeVersion(version) {
  capture(`npm version ${version} --prefix javascript --no-git-tag-version`) //npm's command, which pnpm hands its own version command to anyway
  capture(`uv version ${version} --directory python --offline --no-sync`) //sets pyproject.toml and updates uv.lock
  let manifest = join(root, 'rust/Cargo.toml') //cargo has no command that sets a version, so its line is replaced here, and cargo updates its lockfile to match
  let text = readFileSync(manifest, 'utf8')
  writeFileSync(
    manifest,
    text.replace(/^version = ".+"$/m, `version = "${version}"`),
  )
  capture('cargo update --manifest-path rust/Cargo.toml --workspace --offline')
}

//wait for a workflow's run on a commit to finish, showing its progress, and stop the release if it failed
async function watchRun(workflow, commit, failure) {
  let id = await findRun(workflow, commit)
  if (!succeeds(`gh run watch ${id} --compact --exit-status`))
    throw new Error(
      `${failure}: https://github.com/zootella/ergo/actions/runs/${id}`,
    )
}

//the id of a workflow's latest run on a commit, waiting up to a minute for GitHub to start one after a push
async function findRun(workflow, commit) {
  for (let i = 0; i < 30; i++) {
    let found = capture(
      `gh run list --workflow ${workflow} --commit ${commit} --limit 1 --json databaseId`,
    )
    let runs = JSON.parse(found)
    if (runs.length > 0) return runs[0].databaseId
    await sleep(2000)
  }
  throw new Error(`GitHub started no ${workflow} run on ${commit}`)
}

/*
Three ways to run a command, each from the root. Each takes a command line as text, split on its spaces, followed by any arguments that hold spaces of their own, like a commit message, so a call reads like the command it runs.
*/

//run a command and return what it printed, trimmed, which also keeps its output out of the terminal; failing throws with the command's own complaint
function capture(line, ...more) {
  let [command, ...args] = line.split(' ')
  let options = { cwd: root, encoding: 'utf8', stdio: 'pipe' }
  return execFileSync(command, [...args, ...more], options).trim()
}
//run a command with its output in the terminal; failing throws
function run(line, ...more) {
  let [command, ...args] = line.split(' ')
  execFileSync(command, [...args, ...more], { cwd: root, stdio: 'inherit' })
}
//run a command with its output in the terminal, and say whether it succeeded, for a command whose failure is an answer rather than trouble
function succeeds(line, ...more) {
  let [command, ...args] = line.split(' ')
  let options = { cwd: root, stdio: 'inherit' }
  return spawnSync(command, [...args, ...more], options).status == 0
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})

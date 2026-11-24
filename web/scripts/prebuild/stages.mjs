// What each prebuild stage (web/scripts/<name>.mjs) reads and writes, as
// repo-relative paths (prebuild/cache.mjs has the syntax). prebuild.mjs skips
// a stage when these are unchanged and fails one that reads a repo file not
// listed here. `always` stages depend on something outside the repo (git
// HEAD, the network) and always run.
// crates' build scripts read spec/ (the lint tables, for one), so it is a compiler input.
const compiler = ['crates', 'std', 'spec', 'Cargo.toml', 'Cargo.lock'];

export const stages = [
  { name: 'wasm', inputs: [...compiler, 'scripts/build-wasm.sh', 'rust-toolchain.toml'], outputs: ['dist/wasm', 'web/public/wasm'] },
  { name: 'wasm-size', always: true, inputs: ['dist/wasm', '.wasm-size', 'scripts/build-wasm.sh'], outputs: [], env: ['WASM_SIZE_HISTORY', 'GITHUB_ACTIONS'] },
  { name: 'lint-inputs', inputs: [...compiler, 'web/public/wasm'], outputs: ['web/content/generated/lint-inputs.json'] },
  { name: 'practice-mutants', inputs: [...compiler, 'web/content/practice', 'web/scripts/practice-mutants-equivalent.txt'], outputs: ['web/.gate/mutants.json'] },
  { name: 'stress-parity', inputs: [...compiler, 'dist/wasm', 'web/public/wasm', '**/*.cp'], outputs: [] },
  { name: 'stamp', always: true, inputs: [], outputs: ['web/public/version.json'] },
  { name: 'releases', inputs: ['CHANGELOG.md'], outputs: ['web/content/docs/releases.mdx'] },
  { name: 'practice', inputs: ['web/content/practice', 'web/content/learn/practice', '!web/content/learn/practice/index.mdx', '!web/content/learn/practice/meta.json'], outputs: ['web/content/learn/practice/index.mdx', 'web/content/learn/practice/meta.json'] },
  { name: 'downloads', always: true, inputs: [], outputs: ['web/generated/downloads.json'] },
  { name: 'gen-ref', inputs: [...compiler, 'spec', 'web/content/learn/language'], outputs: ['web/content/docs/ref', 'web/scripts/doc-examples-failing.txt'] },
  { name: 'gen-tools', inputs: compiler, outputs: ['web/content/generated/cli.json'] },
  { name: 'gen-debug', inputs: [...compiler, 'web/content/landing/debug', 'tools/trace'], outputs: ['web/content/generated/stress.json', 'web/public/trace-demo.html', 'web/public/trace.html'] },
  { name: 'facts', inputs: ['Cargo.toml', 'CHANGELOG.md', 'spec', 'web/content', 'web/app', 'web/components', 'web/lib', '!web/lib/facts.json'], outputs: ['web/lib/facts.json'] },
  { name: 'gate', inputs: [...compiler, 'web/content', 'examples', 'corpus/problems', 'dist/wasm', 'web/public/wasm'], outputs: ['web/.gate'] },
];

export const chains = [
  ['wasm', 'wasm-size', 'lint-inputs', 'practice-mutants'],
  ['stamp', 'releases', 'practice', 'downloads', 'gen-ref', 'gen-tools', 'gen-debug', 'facts', 'gate'],
];
// Run after the chains, one at a time: stress-parity holds the browser build
// to a wall-clock limit, which another chain's load on the runner would break.
export const alone = ['stress-parity'];

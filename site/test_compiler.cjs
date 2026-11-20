// Exercise the generated site's gzip loader, real WASM and worker protocol.
// Node's worker_threads only supplies the browser Worker transport here.
const {test, after} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const {Worker} = require("node:worker_threads");
const blobs = new Map();
const urls = Object.assign(class extends URL {}, {
  createObjectURL(blob) { const id = `blob:test-${blobs.size}`; blobs.set(id, blob); return id; },
  revokeObjectURL(id) { blobs.delete(id); }
});
class BrowserWorker {
  constructor(url) {
    this.ready = blobs.get(url).text().then(source => {
      this.worker = new Worker(`const {parentPort} = require('node:worker_threads');
        globalThis.self = globalThis;
        self.location = {href:'http://localhost/worker.js'};
        self.postMessage = data => parentPort.postMessage(data);
        ${source}
        parentPort.on('message', data => self.onmessage({data}));`, {eval:true});
      this.worker.on("message", data => this.onmessage({data}));
      this.worker.on("error", error => this.onerror({message:error.message, preventDefault(){}}));
      return this.worker;
    });
  }
  postMessage(data) { this.ready.then(worker => worker.postMessage(data)); }
  terminate() { this.ready.then(worker => worker.terminate()); }
}
const listeners = new Map();
const context = {WebAssembly, Uint8Array, atob, Blob, Response, DecompressionStream,
  URL:urls, Worker:BrowserWorker, setTimeout, clearTimeout,
  addEventListener(name, callback) { listeners.set(name, callback); }};
vm.runInNewContext(fs.readFileSync(path.join(__dirname, "_build/assets/compiler.js"), "utf8"), context);
const compiler = context.CpCompiler;
after(() => compiler.stop());

test("the generated runtime executes and builds all examples, inline course programs, playground presets, and course predictions in real WASM", async () => {
  const manifests = ["foundations", "advanced"].flatMap(name => JSON.parse(fs.readFileSync(path.join(__dirname, `course/${name}.json`))));
  const builtFixtures = new Set();
  for (const lesson of manifests) for (const exercise of lesson.exercises) {
    if (exercise.kind !== "output") continue;
    const source = fs.readFileSync(path.join(__dirname, `examples/${exercise.fixture}.cp`), "utf8");
    if (!builtFixtures.has(exercise.fixture)) {
      const built = await compiler.call("build", source);
      assert.equal(built.ok, true, `${exercise.fixture} build: ${built.stderr}`);
      assert.match(built.cpp, /int main\(/, exercise.fixture);
      builtFixtures.add(exercise.fixture);
    }
    for (const {input, expected} of exercise.tests) {
      const result = await compiler.call("run", source, input);
      assert.equal(result.ok, true, `${exercise.id}: ${result.stderr}`);
      assert.equal(result.stdout.trim(), expected.trim(), exercise.id);
    }
  }
  // Verify cached @memo executes fast at n=35 (would time out without caching).
  const memoSource = fs.readFileSync(path.join(__dirname, "examples/memo_dp.cp"), "utf8");
  const memo35 = await compiler.call("run", memoSource, "35\n");
  assert.equal(memo35.ok, true, memo35.stderr);
  assert.equal(memo35.stdout.trim(), "14930352");

  // Verify every complete inline runnable course program (excluding explicit not-yet-in-WASM blocks).
  const inlineCases = {
    "max-and-evens": [["5\n-3 8 -4 7 8\n", "8 3"]],
    "range-sums": [["5 3\n3 1 4 1 5\n1 5\n2 3\n4 4\n", "14\n5\n1"]],
    "best-window": [
      ["5 2\n3 -1 4 1 5\n", "6"],
      ["5 5\n3 -1 4 1 5\n", "12"],
      ["3 1\n-2 -9 -4\n", "-2"],
    ],
    "grid-sum": [["2 3\n", "21 21 YES"]],
  };
  const inlineFound = new Map();
  for (const file of fs.readdirSync(path.join(__dirname, "content")).filter(f => f.startsWith("course-") && f.endsWith(".md"))) {
    const md = fs.readFileSync(path.join(__dirname, "content", file), "utf8");
    for (const match of md.matchAll(/```cp Prototype tested · ([a-z-]+)\.cp\n([\s\S]*?)```/g)) {
      inlineFound.set(match[1], match[2]);
    }
  }
  assert.deepEqual([...inlineFound.keys()].sort(), Object.keys(inlineCases).sort());
  for (const [name, source] of inlineFound) {
    const built = await compiler.call("build", source);
    assert.equal(built.ok, true, `${name} build: ${built.stderr}`);
    for (const [input, expected] of inlineCases[name]) {
      const result = await compiler.call("run", source, input);
      assert.equal(result.ok, true, `${name}: ${result.stderr}`);
      assert.equal(result.stdout.trim(), expected.trim(), name);
    }
  }

  const pgHtml = fs.readFileSync(path.join(__dirname, "_build/playground.html"), "utf8");
  const m = pgHtml.match(/<script id="playground-examples" type="application\/json">(.*?)<\/script>/s);
  assert.ok(m, "playground-examples payload missing");
  const presets = JSON.parse(m[1]);
  const expectedOutputs = {
    sum: "14",
    early_return: "11",
    multitest_divider: "0\n1 10\n5 10\n7 10",
    bounds: "14",
    knapsack: "9",
    range_array: "1 1 3 0 1",
    "str-scan": "5 11",
    lends: "6 1 -1",
    tropical_dp: "3",
    optional_coalesce: "6 -4",
    memo_dp: "89",
    segtree_monoid: "8\n17",
    lazy_action: "19\n11",
    colocated_brute: "6",
    accumulation: "6",
    ascii: "3",
    optional: "8",
    memo: "45",
  };
  assert.equal(presets.length, Object.keys(expectedOutputs).length);
  for (const preset of presets) {
    const built = await compiler.call("build", preset.source);
    assert.equal(built.ok, true, `${preset.id} build: ${built.stderr}`);
    const result = await compiler.call("run", preset.source, preset.stdin);
    assert.equal(result.ok, true, `${preset.id}: ${result.stderr}`);
    assert.equal(result.stdout.trim(), expectedOutputs[preset.id], preset.id);
  }
});

test("arbitrary edited programs, stdin, C++17, diagnostics and hover use the real compiler", async () => {
  const source = "let n: Int = read()\nprint(n * 7)\n";
  assert.equal((await compiler.call("run", source, "6")).stdout, "42\n");
  assert.match((await compiler.call("build", source)).cpp, /int main\(/);
  const checked = await compiler.call("check", "print(missing)\n");
  assert.equal(checked.ok, false);
  assert.match(checked.diagnostics[0].message, /missing/);
  assert.equal(checked.diagnostics[0].start, 6);
  assert.match((await compiler.call("hover", source, source.indexOf("n *"))).text, /Int/);
});

test("0.2.2 Tropical Optionals and deleted-form migration diagnostics run in WASM", async () => {
  const coinChange = [
    "let (n, x): (1..=100, 1..=1e6) = read()",
    "let coins: [0..n]Int = read()",
    "",
    "var dp: [0..=x]?Int = none",
    "dp[0] = 0",
    "for s in 1..=x:",
    "    for c in coins:",
    "        if c <= s: dp[s] min= dp[s - c] + 1",
    "print(dp[x] ?? -1)",
    "",
  ].join("\n");
  const okRun = await compiler.call("run", coinChange, "3 11\n1 5 7\n");
  assert.equal(okRun.ok, true, okRun.stderr);
  assert.equal(okRun.stdout, "3\n");
  const missRun = await compiler.call("run", coinChange, "2 3\n2 4\n");
  assert.equal(missRun.ok, true, missRun.stderr);
  assert.equal(missRun.stdout, "-1\n");
  const built = await compiler.call("build", coinChange);
  assert.equal(built.ok, true, built.stderr);
  assert.match(built.cpp, /std::optional<ll>/);
  assert.match((await compiler.call("hover", coinChange, coinChange.indexOf("dp[0]"))).text, /\?Int/);

  const deferCheck = await compiler.call("check", "defer print(1)\n");
  assert.equal(deferCheck.ok, false);
  assert.equal(deferCheck.diagnostics[0].code, "E0503");
  assert.match(deferCheck.diagnostics[0].message, /removed in CP Lang (?:v2\.2|0\.2\.2)/);

  const optPlusEq = await compiler.call("check", "var x: ?Int = 1\nx += 2\n");
  assert.equal(optPlusEq.ok, false);
  assert.equal(optPlusEq.diagnostics[0].code, "E0309");
  assert.match(optPlusEq.diagnostics[0].message, /\?Int/);
});

test("a runaway program times out without poisoning the next run", async () => {
  await assert.rejects(compiler.call("run", "var x = 0\nwhile true:\n    x = 1 - x\n", ""), /Stopped after 5 seconds/);
  assert.equal((await compiler.call("run", "print(43)\n", "")).stdout, "43\n");
});

test("Stop cancels a worker and the next call restarts it", async () => {
  const running = compiler.call("run", "var x = 0\nwhile true:\n    x = 1 - x\n", "");
  setTimeout(() => compiler.stop(), 50);
  await assert.rejects(running, /Program stopped/);
  assert.equal((await compiler.call("run", "print(44)\n", "")).stdout, "44\n");
});

test("a page restored from back-forward cache creates a fresh worker URL", async () => {
  assert.equal((await compiler.call("run", "print(44)\n", "")).stdout, "44\n");
  listeners.get("pagehide")();
  assert.equal(blobs.size, 0);
  assert.equal((await compiler.call("run", "print(45)\n", "")).stdout, "45\n");
});

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

test("the generated runtime executes all examples, playground presets, and course predictions in real WASM", async () => {
  const manifests = ["foundations", "advanced"].flatMap(name => JSON.parse(fs.readFileSync(path.join(__dirname, `course/${name}.json`))));
  for (const lesson of manifests) for (const exercise of lesson.exercises) {
    if (exercise.kind !== "output") continue;
    const source = fs.readFileSync(path.join(__dirname, `examples/${exercise.fixture}.cp`), "utf8");
    for (const {input, expected} of exercise.tests) {
      const result = await compiler.call("run", source, input);
      assert.equal(result.ok, true, result.stderr);
      assert.equal(result.stdout.trim(), expected.trim(), exercise.id);
    }
  }
  const pgHtml = fs.readFileSync(path.join(__dirname, "_build/playground.html"), "utf8");
  const m = pgHtml.match(/<script id="playground-examples" type="application\/json">(.*?)<\/script>/s);
  assert.ok(m, "playground-examples payload missing");
  const presets = JSON.parse(m[1]);
  const expectedOutputs = {
    sum: "14",
    bounds: "14",
    knapsack: "9",
    range_array: "1 1 3 0 1",
    "str-scan": "5 11",
    accumulation: "6",
    ascii: "3",
    optional: "8",
    memo: "45",
  };
  assert.equal(presets.length, Object.keys(expectedOutputs).length);
  for (const preset of presets) {
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

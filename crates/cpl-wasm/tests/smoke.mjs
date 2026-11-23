// Runs the built WebAssembly module through js/cpl.js, in Node: the same
// two files a page loads. `scripts/build-wasm.sh --smoke` builds and then
// runs this; on its own it is
//
//   node crates/cpl-wasm/tests/smoke.mjs [dist/wasm]
//
// What it holds the module to is in the name of each test below. That the
// module agrees with `cpl` program by program is `tests/parity.rs`; this
// file is about what only the real module can show: the interface, the
// limits of a browser's stack, and what happens after a trap.

import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const dist = resolve(process.argv[2] ?? join(root, "dist/wasm"));
const { load, BUILD_WASM, BUILD_CALLS, CplError } = await import(pathToFileURL(join(dist, "cpl.js")));
// cpl-build.wasm has every method, so the tests below use it; the first
// test holds cpl.wasm to the same answers without the build methods.
const cpl = await load(BUILD_WASM);
const core = await load();

test("cpl.wasm runs what cpl-build.wasm does, without build, explain or stress", () => {
  const program = "let n: Int = read()\nprint(n * 2)\n";
  assert.equal(core.run(program, "21").stdout, cpl.run(program, "21").stdout);
  assert.deepEqual(core.check(program), cpl.check(program));
  assert.deepEqual(core.lint(program), cpl.lint(program));
  for (const method of BUILD_CALLS) {
    assert.throws(() => core[method](program), (error) => error instanceof CplError && error.kind === "load");
  }
  assert.equal(core.run(program, "4").stdout, "8\n");
});

const DOUBLE = "let n: Int = read()\nprint(n * 2)\n";
const recurse = (depth) =>
  `fn down(n: Int) -> Int:\n    if n == 0:\n        return 0\n    down(n - 1) + 1\nprint(down(${depth}))\n`;
// A depth-first search down a path of `n` nodes, with the call a few
// statements deep, so that each call takes more of the engine's stack.
const search = (n) => `let n: 1..=100000 = ${n}
domain V = 1..=n
var adj: [V]List<V> = []
for i in 1..n:
    adj[V(i)].push(V(i + 1))
    adj[V(i + 1)].push(V(i))
var seen: [V]Bool = false
var count = 0
fn visit(u: V):
    seen[u] = true
    count += 1
    for v in adj[u]:
        if not seen[v]:
            if count > 0:
                match count % 1:
                    0 => visit(v)
                    _ => print("unreachable")
print("searching")
visit((V.first)!)
print(count)
`;

test("the module says which toolchain it is", () => {
  const manifest = readFileSync(join(root, "Cargo.toml"), "utf8");
  assert.equal(cpl.version, /^version = "(.+)"$/m.exec(manifest)[1]);
});

test("replay runs one recorded call again with its fn edited", () => {
  const source = "var total = 0\nfn add(i: Int) -> Int:\n    total += i\n    return total\nfor i in 1..=3:\n    print(add(i))\n";
  const { trace } = cpl.trace(source, "");
  const entry = trace.runs.fast.steps.findIndex((step) => step.frames?.[0]?.fn === "add" && step.values.total === 3);
  assert.ok(entry > 0, "the second call has a step of its own");
  const edited = source.replace("return total", "return total * 10");
  const report = cpl.replay(source, edited, "", entry);
  assert.equal(report.call.name, "add");
  assert.equal(report.call.ordinal, 2);
  assert.deepEqual(report.returned, { before: 3, after: 30 });
  assert.equal(report.fallback, null);
  assert.ok(report.change, "the call returns something else");
  assert.match(cpl.replay(source, "fn (", "", entry).error, /edited file does not build/);
});

test("a run that never ends stops as out of time, and the module goes on", () => {
  // The browser has no clock: the limit is a budget of loop rounds and calls.
  const forever = "var i = 0\nwhile i >= 0:\n    i = (i + 1) % 2\nprint(i)\n";
  const late = /took longer than its time limit/;
  assert.match(cpl.run(forever).diagnostics[0].message, late);
  assert.match(cpl.trace(forever).diagnostics[0].message, late);
  const stress = `let n: 1..=5 = read()\n${forever.replace("print(i)", "print(n)")}\n@alt(program)\nfn brute():\n    let n: 1..=5 = read()\n    print(n)\n`;
  const stressed = cpl.stress(stress, { cases: 3 });
  assert.equal(stressed.status, "fail");
  assert.equal(stressed.counterexample.verdict, "TLE");
  const source = "var total = 0\nfn add(i: Int) -> Int:\n    total += i\n    return total\nfor i in 1..=3:\n    print(add(i))\n";
  const entry = cpl.trace(source, "").trace.runs.fast.steps.findIndex((step) => step.frames?.[0]?.fn === "add");
  const edited = source.replace("return total", "while total >= 0:\n        total = (total + 1) % 2\n    return total");
  assert.match(cpl.replay(source, edited, "", entry).trap, late);
  assert.equal(cpl.run(DOUBLE, "21\n").stdout, "42\n");
});

test("trace records a live program for the trace viewer", () => {
  const source = [
    "let n: 1..=10 = read()",
    "let a: [0..n]1..=100 = read()",
    "var best = 0",
    "for i in 0..n:",
    "    if i > 0: best max= a[i]",
    "    debug(i, best)",
    "print(best)",
    "",
    "@alt(program)",
    "fn slow():",
    "    var m = 0",
    "    for x in a: m = max(m, x)",
    "    print(m)",
    "",
  ].join("\n");
  const run = cpl.trace(source, "3\n9 5 2\n");
  assert.equal(run.ok, true);
  assert.equal(run.stdout, "5\n");
  assert.equal(run.debug, "[line 6] i = 0, best = 0\n[line 6] i = 1, best = 5\n[line 6] i = 2, best = 5\n");
  const { trace } = run;
  assert.equal(trace.schemaVersion, 1);
  assert.equal(trace.source.code, source);
  assert.equal(trace.variables.a.kind, "array");
  // A step is a snapshot of the declared variables in scope at it.
  for (const side of ["fast", "slow"]) {
    for (const step of trace.runs[side].steps) {
      for (const id of Object.keys(step.values)) assert.ok(id in trace.variables, id);
    }
    assert.deepEqual(trace.runs[side].steps.at(-1).values.a, [9, 5, 2]);
  }
  // The planted bug: the two runs meet at the print and differ there.
  const printed = (side) => trace.runs[side].steps.find((step) => step.checkpoint === "print 1").values.stdout;
  assert.deepEqual(printed("fast"), ["5"]);
  assert.deepEqual(printed("slow"), ["9"]);
  assert.ok(!JSON.stringify(trace).includes("<"));

  const broken = cpl.trace("print(nope)\n");
  assert.equal(broken.ok, false);
  assert.equal(broken.trace, null);
});

test("run reads stdin and prints", () => {
  const run = cpl.run(DOUBLE, "21\n");
  assert.deepEqual(
    { ok: run.ok, exitCode: run.exitCode, stdout: run.stdout, stderr: run.stderr },
    { ok: true, exitCode: 0, stdout: "42\n", stderr: "" },
  );
  assert.deepEqual([...run.stdoutBytes], [0x34, 0x32, 0x0a]);
  assert.deepEqual(run.diagnostics, []);
});

test("run takes stdin as bytes too", () => {
  assert.equal(cpl.run(DOUBLE, new TextEncoder().encode("4")).stdout, "8\n");
});

test("every program of cpl's own tests prints what its .out file says", () => {
  const dir = join(root, "crates/cpl-cli/tests/programs");
  const programs = readdirSync(dir).filter((file) => file.endsWith(".cp"));
  assert.ok(programs.length >= 10, `only ${programs.length} programs in ${dir}`);
  const read = (file) => {
    try {
      return readFileSync(join(dir, file));
    } catch {
      return new Uint8Array();
    }
  };
  for (const program of programs) {
    const stem = program.slice(0, -".cp".length);
    // A program with its `@interactor` is played against it by `cpl run`
    // on two threads, which a page does not have: its .out is the
    // conversation, not what the solution prints alone.
    if (read(program).toString().includes("\n@interactor")) continue;
    const run = cpl.run(read(program).toString(), read(`${stem}.in`), { name: program });
    assert.deepEqual(Buffer.from(run.stdoutBytes), read(`${stem}.out`), program);
  }
});

test("check reports an error as data and as text", () => {
  const source = "let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nprint(a[n])\n";
  const check = cpl.check(source, { name: "sum.cp" });
  assert.equal(check.ok, false);
  assert.equal(check.exitCode, 1);
  assert.equal(check.diagnostics.length, 1);
  const [error] = check.diagnostics;
  assert.equal(error.code, "E0301");
  assert.equal(error.severity, "error");
  assert.equal(source.slice(error.start, error.end), "n");
  assert.deepEqual(error.range.start, { line: 3, character: 8 });
  assert.match(error.help, /V\(n\)/);
  assert.match(check.stderr, /^error\[E0301\]/);
  assert.match(check.stderr, /--> sum\.cp:4:9/);
});

test("lint reports what `cpl lint` warns about, kept apart from check", () => {
  const source = "let xs: List<Int> = [1, 4, 7]\nvar found = false\nfor x in xs:\n    if x > 3:\n        found = true\n        break\nprint(yes(found))\n";
  assert.equal(cpl.check(source).diagnostics.length, 0);
  const lint = cpl.lint(source);
  assert.equal(lint.exitCode, 0);
  assert.deepEqual(lint.diagnostics.map((d) => [d.code, d.severity]), [["L0501", "warning"]]);
  assert.match(lint.stderr, /warning\[L0501\]/);
});

test("a program with errors does not run or build", () => {
  const run = cpl.run("print(total)\n");
  assert.deepEqual([run.ok, run.exitCode, run.stdout], [false, 1, ""]);
  const build = cpl.build("print(total)\n");
  assert.deepEqual([build.ok, build.exitCode, build.cpp, build.bytes], [false, 1, null, 0]);
});

test("positions index the JavaScript string, not its UTF-8 bytes", () => {
  const source = "// é😀\nprint(total)\n";
  const [error] = cpl.check(source).diagnostics;
  assert.equal(source.slice(error.start, error.end), "total");
  assert.deepEqual(error.range.start, { line: 1, character: 6 });
});

test("build writes C++17 with the source in its header", () => {
  const build = cpl.build(DOUBLE, { name: "double.cp" });
  assert.equal(build.ok, true);
  assert.match(build.cpp, /^\/\/ double\.cp: C\+\+17 written by cpl/);
  assert.match(build.cpp, /\/\/     print\(n \* 2\)/);
  assert.match(build.cpp, /int main\(/);
  assert.equal(build.bytes, Buffer.byteLength(build.cpp));
});

test("a failed run keeps what it printed and says where it failed", () => {
  const run = cpl.run("print(1)\nlet n: Int = read()\nassert n > 0, \"{n} is not positive\"\n", "-5");
  assert.deepEqual([run.ok, run.exitCode, run.stdout], [false, 3, "1\n"]);
  assert.equal(run.diagnostics.length, 1);
  assert.equal(run.diagnostics[0].code, "R0401");
  assert.equal(run.diagnostics[0].range.start.line, 2);
  assert.match(run.stderr, /-5 is not positive/);
});

test("hover, definition and symbols answer at an offset", () => {
  const source = "fn double(x: Int) -> Int:\n    x * 2\nlet n: Int = read()\nprint(double(n))\n";
  const call = source.lastIndexOf("double");
  assert.match(cpl.hover(source, call).text, /double/);
  assert.equal(cpl.definition(source, call).start, source.indexOf("double"));
  assert.equal(cpl.hover(source, 0), null);
  assert.deepEqual(
    cpl.symbols(source).map(({ kind, name }) => [kind, name]),
    [["fn", "double"], ["value", "n"]],
  );
});

test("the standard library is inside the module: `use std.*` needs no files", () => {
  const sorted = cpl.run(
    "use std.algebra.{desc}\nvar xs: List<Int> = [3, 1, 2]\nsort(mut xs, by: desc)\nprint(xs)\n",
  );
  assert.deepEqual([sorted.ok, sorted.stdout, sorted.stderr], [true, "3 2 1\n", ""]);
  const choose = cpl.run(
    "use std.mod.{Mod, Comb}\nlet c = Comb<Mod<1000000007>>()\nprint(c.choose(5, 2))\n",
  );
  assert.deepEqual([choose.ok, choose.stdout], [true, "10\n"]);
  assert.equal(cpl.check("use std.segtree.{SegTree}\nprint(1)\n").ok, true);
  assert.match(cpl.check("use std.nothing.{X}\n").stderr, /E0206/);
});

test("lsp is the language server without the stdio framing", () => {
  const [reply] = cpl.lsp({ jsonrpc: "2.0", id: 1, method: "initialize", params: {} });
  assert.equal(reply.id, 1);
  assert.ok(reply.result.capabilities);
  const uri = "file:///main.cp";
  const sent = cpl.lsp({
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: { textDocument: { uri, languageId: "cp", version: 1, text: "print(total)\n" } },
  });
  const published = sent.find((message) => message.method === "textDocument/publishDiagnostics");
  assert.equal(published.params.uri, uri);
  assert.equal(published.params.diagnostics.length, 1);
});

test("recursion a browser's stack can hold runs", () => {
  assert.equal(cpl.run(recurse(300)).stdout, "300\n");
  assert.equal(cpl.run(search(100)).stdout, "searching\n100\n");
});

test("twenty million loop rounds finish inside the run limit", () => {
  // The browser's run limit is a budget of rounds, not a clock: it holds
  // the VM's speed, so what runs in about a second natively runs here too.
  const run = cpl.run("var s = 0\nfor i in 0..20_000_000:\n    s += i % 7\nprint(s)\n");
  assert.deepEqual([run.exitCode, run.stdout], [0, "59999997\n"]);
});

test("recursion that never ends is a diagnostic, not a crash", () => {
  const run = cpl.run("fn forever(n: Int) -> Int:\n    forever(n + 1)\nprint(forever(0))\n");
  assert.equal(run.exitCode, 3);
  assert.match(run.diagnostics[0].message, /`forever` never stops calling itself\?/);
  assert.equal(run.diagnostics[0].range.start.line, 1);
});

test("recursion too deep for the engine's stack is that diagnostic too", () => {
  // Each call here is heavy enough that the engine's stack fills before the
  // module's limit on the stack calls take: the run is tried again under a
  // lower one.
  const source = search(5000);
  const run = cpl.run(source);
  assert.deepEqual([run.exitCode, run.stdout], [3, "searching\n"]);
  const [failure] = run.diagnostics;
  assert.equal(failure.message, "the stack ran out: `visit` never stops calling itself?");
  assert.equal(source.slice(failure.start, failure.end), "visit(v)");
  assert.ok(run.maxStack >= 16 << 10 && run.maxStack < 3 << 20, `stopped at ${run.maxStack}`);
  assert.equal(cpl.run(DOUBLE, "21").stdout, "42\n");
});

test("a trap is a CplError, and the next call starts clean", () => {
  // Nesting this deep in one expression overflows the engine's stack inside
  // the parser, where no limit of the interpreter's applies.
  const nested = `print(${"(".repeat(200_000)}1${")".repeat(200_000)})\n`;
  const ranOutOfStack = (error) =>
    error instanceof CplError && error.kind === "stack" && /ran out of stack/.test(error.message);
  assert.throws(() => cpl.check(nested), ranOutOfStack);
  assert.throws(() => cpl.run(nested), ranOutOfStack);
  assert.equal(cpl.run(DOUBLE, "21").stdout, "42\n");
});

test("a second edit of the same file is checked again", () => {
  assert.equal(cpl.check("print(total)\n").ok, false);
  assert.equal(cpl.check("let total = 1\nprint(total)\n").ok, true);
  assert.equal(cpl.check("print(total)\n").ok, false);
});

test("a loop of a few million steps finishes in seconds", () => {
  const started = performance.now();
  const run = cpl.run("var s = 0\nfor i in 0..3000000:\n    s += i % 7\nprint(s)\n");
  assert.equal(run.stdout, "8999994\n");
  const ms = performance.now() - started;
  assert.ok(ms < 5_000, `took ${ms.toFixed(0)} ms`);
});

test("stress finds and shrinks a planted bug in wasm with first differing value", () => {
  const buggy = [
    "let n: 1..=10 = read()",
    "let a: [0..n]1..=100 = read()",
    "var best = 0",
    "for i in 0..n:",
    "    if i > 0: best max= a[i]",
    "print(best)",
    "",
    "@alt(program)",
    "fn slow():",
    "    var best = 0",
    "    for x in a: best max= x",
    "    print(best)",
    "",
  ].join("\n");

  const fail = cpl.stress(buggy, { cases: 50, seed: 1, size: 6 });
  assert.equal(fail.status, "fail");
  assert.equal(fail.counterexample.case, 2);
  assert.equal(fail.counterexample.seed, 2);
  assert.equal(fail.counterexample.raw_input, "1\n1\n");
  assert.equal(fail.counterexample.expected, "1\n");
  assert.equal(fail.counterexample.got, "0\n");
  assert.equal(fail.counterexample.verdict, "WA");
  assert.equal(fail.counterexample.saved, null);
  assert.equal(fail.counterexample.explain.divergence.name, "best");
  assert.equal(fail.counterexample.explain.divergence.nth, 2);
  assert.equal(fail.counterexample.explain.divergence.fast, null);
  assert.deepEqual(fail.counterexample.explain.divergence.brute, { line: 11, value: "1" });

  const fixed = buggy.replace("if i > 0: best max= a[i]", "best max= a[i]");
  const ok = cpl.stress(fixed, { cases: 25, seed: 1, size: 6 });
  assert.deepEqual(ok, { status: "ok", cases: 25 });
});


test("sample makes an input a program's read() types accept", () => {
  const program = "let n: 1..=5 = read()\nlet a: [0..n]1..=9 = read()\nprint(a.len)\n";
  const { input } = cpl.sample(program, { seed: 3, size: 4 });
  const [n, ...a] = input.trim().split(/\s+/).map(Number);
  assert.ok(n >= 1 && n <= 4 && a.length === n && a.every((x) => x >= 1 && x <= 9), input);
  assert.equal(cpl.run(program, input).stdout, `${n}\n`);
  assert.deepEqual(cpl.sample(program, { seed: 3, size: 4 }), { input });
});

test("the worker client has every method the worker answers", () => {
  const methods = (file) => /METHODS = (?:new Set\()?\[([^\]]+)\]/.exec(readFileSync(join(dist, file), "utf8"))[1];
  assert.equal(methods("cpl.js"), methods("cpl.worker.js"));
});

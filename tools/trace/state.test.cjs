// Focused tests for the site trace debugger: pure core logic, transport state
// machine and keyboard map. Run: node --test tools/trace/state.test.cjs
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const {core:T, section} = require("./read-viewer.cjs");

const samples = Object.fromEntries(["knapsack", "dijkstra", "tree", "grid", "array"].map(name =>
  [name, JSON.parse(fs.readFileSync(path.join(__dirname, "./samples", `${name}.json`), "utf8"))]));
const plain = value => JSON.parse(JSON.stringify(value));

test("every sample validates and analyzes", () => {
  for (const trace of Object.values(samples)) { T.validate(trace); T.analyze(trace); }
});

test("knapsack first divergence is fast step 5 vs spec step 9 at dp[1][4]", () => {
  const trace = samples.knapsack, analysis = T.analyze(trace), s = T.summary(trace, analysis);
  assert.equal(s.state, "diverge");
  assert.equal(s.first.fastIndex, 4);
  assert.equal(s.first.slowIndex, 8);
  assert.deepEqual(plain(s.difference), {path:[1, 4], fast:6, slow:3, fastPresent:true, slowPresent:true, variable:"dp"});
  assert.equal(s.title, "First divergence at step 5");
  assert.match(s.detail, /^dp\[1\]\[4\]: fast 6 ≠ spec 3 · checkpoint cell:1:4$/);
});

test("a --from window numbers its steps as the whole run does", () => {
  const window = structuredClone(samples.dijkstra);
  window.runs.fast.firstStep = 1000;
  T.validate(window);
  assert.equal(T.stepNumber(window.runs.fast, 0), 1001);
  assert.match(T.summary(window, T.analyze(window)).detail, /steps from step 1001 ·/);
  window.runs.fast.firstStep = 0;
  assert.throws(() => T.validate(window), /firstStep must be a positive integer/);
});

test("summary covers single-run, agreeing and incomparable traces", () => {
  const single = samples.dijkstra;
  assert.equal(T.summary(single, T.analyze(single)).state, "single");
  const agree = structuredClone(samples.knapsack);
  agree.runs.fast.steps = agree.runs.fast.steps.slice(0, 4);
  assert.equal(T.summary(agree, T.analyze(agree)).state, "agree");
  const none = structuredClone(samples.knapsack);
  none.runs.slow.steps.forEach(s => { delete s.checkpoint; });
  assert.equal(T.summary(none, T.analyze(none)).state, "incomparable");
});

test("summary names the trap of a single run that failed", () => {
  const trapped = structuredClone(samples.dijkstra);
  const steps = trapped.runs.fast.steps, last = steps[steps.length - 1];
  last.label = "failed: index 3 is outside an array dimension of length 3";
  const s = T.summary(trapped, T.analyze(trapped));
  assert.equal(s.state, "trapped");
  assert.equal(s.title, `Trapped at step ${steps.length}, line ${last.line}`);
  assert.equal(s.detail, "index 3 is outside an array dimension of length 3");
  assert.equal(s.trap, steps.length - 1);
});

test("spec lane holds the last aligned checkpoint between shared checkpoints", () => {
  const trace = structuredClone(samples.knapsack);
  delete trace.runs.fast.steps[5].checkpoint;
  const analysis = T.analyze(trace);
  const held = T.referenceAt(analysis, 5);
  assert.equal(held.exact, false);
  assert.equal(held.pair.fastIndex, 4);
  assert.equal(T.referenceAt(analysis, 4).exact, true);
  const before = structuredClone(trace);
  before.runs.fast.steps[0].checkpoint = "only-fast";
  assert.equal(T.referenceAt(T.analyze(before), 0).pair, null);
});

test("divergence navigation walks forward and back and stops at the ends", () => {
  const analysis = T.analyze(samples.knapsack), list = analysis.divergences.map(p => p.fastIndex);
  assert.equal(T.adjacentDivergence(analysis, -1, 1), list[0]);
  assert.equal(T.adjacentDivergence(analysis, list[0], 1), list[1]);
  assert.equal(T.adjacentDivergence(analysis, list[1], -1), list[0]);
  assert.equal(T.adjacentDivergence(analysis, list[0], -1), null);
  assert.equal(T.adjacentDivergence(analysis, list.at(-1), 1), null);
});

test("changes lists leaf writes relative to the previous snapshot", () => {
  const run = samples.knapsack.runs.fast;
  assert.deepEqual(T.changes(run, 0, ["dp"]), []);
  assert.deepEqual(plain(T.changes(run, 4, ["dp", "answer"])), [{variable:"dp", path:[1, 4], before:0, after:6, beforePresent:true, afterPresent:true}]);
  const scoped = {steps:[{values:{}}, {values:{x:1}}]};
  assert.deepEqual(plain(T.changes(scoped, 1, ["x"])), [{variable:"x", path:[], after:1, afterPresent:true, beforePresent:false}]);
});

test("checkpoint navigation skips internal steps without treating them as matches", () => {
  const trace = structuredClone(samples.knapsack);
  delete trace.runs.fast.steps[5].checkpoint;
  const analysis = T.analyze(trace);
  assert.equal(T.adjacentCheckpoint(analysis, 4, 1), 6);
  assert.equal(T.adjacentCheckpoint(analysis, 5, -1), 4);
  assert.equal(T.adjacentCheckpoint(analysis, 0, -1), null);
  assert.equal(T.adjacentCheckpoint(analysis, 33, 1), null);
  assert.equal(T.referenceAt(analysis, 5).exact, false);
});

test("checkpoint picker bounds the DOM while retaining the current neighborhood", () => {
  const pairs = Array.from({length:100000}, (_, i) => ({fastIndex:i * 2}));
  const analysis = {pairs, byFast:new Map(pairs.map(p => [p.fastIndex, p]))};
  const window = T.checkpointWindow(analysis, 150001, 101);
  assert.equal(window.length, 101);
  assert.ok(window.some(p => p.fastIndex === 150000));
  assert.equal(T.referenceAt(analysis, 150001).pair.fastIndex, 150000);
  assert.equal(T.referenceAt(analysis, -1).pair, null);
  assert.equal(T.checkpointWindow({pairs:[]}, 0).length, 0);
});

test("format and valueAt keep false, zero, null and out-of-scope distinct", () => {
  assert.deepEqual([T.format(false), T.format(0), T.format(null), T.format(undefined, false), T.format([1, "a"])], ["false", "0", "none", "out of scope", '[1,"a"]']);
  const values = {dp:[[0, 5]], z:null};
  assert.deepEqual(T.valueAt(values, "dp", [0, 1]), {value:5, present:true});
  assert.deepEqual(T.valueAt(values, "dp", [3, 0]), {value:undefined, present:false});
  assert.deepEqual(T.valueAt(values, "z", []), {value:null, present:true});
  assert.deepEqual(T.valueAt(values, "missing", []), {value:undefined, present:false});
});

test("future debug variables absent from both runs do not create false divergences", () => {
  const trace = structuredClone(samples.knapsack);
  trace.variables.future = {name:"debug(score)", kind:"json", type:"Int"};
  delete trace.compare; // real producers compare every declared variable
  const first = T.summary(trace, T.analyze(trace));
  assert.equal(first.first.fastIndex, 4); // unchanged: first real DP mismatch
  const pair = T.analyze(trace).byFast.get(0);
  assert.deepEqual(pair.differences, []); // both runs have not reached debug(score)
  assert.deepEqual(T.diff(undefined, undefined, [], false, false), []);
  trace.runs.fast.steps[0].values.future = null;
  const differences = T.analyze(trace).byFast.get(0).differences;
  assert.equal(differences.length, 1); // present none vs absent remains meaningful
  assert.equal(differences[0].fastPresent, true);
  assert.equal(differences[0].slowPresent, false);
});

test("tree layout centers parents over children and gives leaves distinct columns", () => {
  const trace = samples.tree, value = trace.runs.fast.steps[0].values.tree;
  const pos = T.layout(value, trace.variables.tree);
  assert.equal(pos.size, value.nodes.length);
  const at = id => pos.get(String(id));
  for (const e of value.edges) assert.ok(at(e.to).y > at(e.from).y);
  const leaves = value.nodes.filter(n => !value.edges.some(e => e.from === n.id)).map(n => at(n.id).x);
  assert.equal(new Set(leaves).size, leaves.length);
  const kids = value.edges.filter(e => e.from === trace.variables.tree.root).map(e => at(e.to).x);
  assert.equal(at(trace.variables.tree.root).x, (Math.min(...kids) + Math.max(...kids)) / 2);
});

test("graph layout keeps explicit coordinates and places the rest on a circle", () => {
  const trace = samples.dijkstra, value = trace.runs.fast.steps[0].values.graph;
  const pos = T.layout(value, trace.variables.graph);
  assert.deepEqual(pos.get("0"), {x:value.nodes[0].x, y:value.nodes[0].y});
  const free = T.layout({nodes:[{id:"a"}, {id:"b"}], edges:[]}, {kind:"graph"});
  assert.notDeepEqual(free.get("a"), free.get("b"));
  assert.equal(T.layout({nodes:[], edges:[]}, {kind:"tree"}).size, 0);
});

test("viewBox pads laid-out vertices and is null for an empty graph or tree", () => {
  assert.equal(T.viewBox(T.layout({nodes:[], edges:[]}, {kind:"graph"})), null);
  assert.equal(T.viewBox(T.layout({nodes:[], edges:[]}, {kind:"tree"})), null);
  assert.deepEqual(T.viewBox(new Map([["a", {x:10, y:20}], ["b", {x:110, y:70}]])), {x:-22, y:-12, w:164, h:114});
  const one = T.viewBox(T.layout({nodes:[{id:0}], edges:[]}, {kind:"tree", root:0}));
  assert.ok(Object.values(one).every(Number.isFinite) && one.w > 0 && one.h > 0);
});

test("live handoff reads the offline parent first, then sessionStorage, only under #live", () => {
  const storage = text => ({getItem:key => key === "cp-live-trace" ? text : null});
  const page = (hash, extra = {}) => { const w = {location:{hash}, ...extra}; w.parent = w; return w; };
  assert.equal(T.liveTrace(page("#live"), storage("{\"a\":1}")), '{"a":1}');
  assert.equal(T.liveTrace(page("#sample=x"), storage("{}")), null);
  assert.equal(T.liveTrace(page("#live"), storage(null)), null);
  assert.equal(T.liveTrace(page("#live"), null), null);
  const shell = {location:{hash:"#trace.html#live"}, cpLiveTrace:"offline"};
  assert.equal(T.liveTrace({parent:shell, location:{hash:""}}, storage("standalone")), "offline");
  const blocked = {location:{hash:""}, get parent() { throw new Error("cross-origin"); }};
  assert.equal(T.liveTrace(blocked, storage("x")), null);
});

test("legend names every renderer state and adds the mismatch swatch only when comparing", () => {
  for (const kind of ["dp", "grid", "array", "graph", "tree"]) {
    assert.ok(T.legend(kind, false).length >= 2);
    assert.deepEqual(T.legend(kind, true).at(-1), ["mismatch", "Fast ≠ spec"]);
    assert.ok(!T.legend(kind, false).some(([cls]) => cls === "mismatch"));
  }
});

test("keyboard map covers transport keys and ignores modified chords", () => {
  const k = (key, extra = {}) => T.keyAction({key, code:key === " " ? "Space" : key, ...extra});
  assert.deepEqual([k("ArrowRight"), k("ArrowLeft"), k("Home"), k("End"), k(" "), k("]"), k("[")],
    ["next", "previous", "first", "last", "toggle", "next-divergence", "previous-divergence"]);
  assert.equal(k("ArrowRight", {shiftKey:true}), "next-divergence");
  assert.equal(k("ArrowRight", {ctrlKey:true}), null);
  assert.equal(k("ArrowLeft", {metaKey:true}), null);
  assert.equal(k("a"), null);
});

function fakeClock() {
  const timers = new Map(); let id = 0;
  return {
    schedule:(fn, ms) => { timers.set(++id, {fn, ms}); return id; },
    cancel:handle => timers.delete(handle),
    tick() { for (const {fn} of [...timers.values()]) fn(); },
    get active() { return [...timers.values()]; },
  };
}

test("player clamps seeks, steps, and reports start/end state", () => {
  const events = [], p = T.createPlayer({count:5, onChange:(s, reason) => events.push([s.index, reason])});
  p.go(99); assert.equal(p.state().index, 4); assert.ok(p.state().atEnd);
  p.go(-3); assert.equal(p.state().index, 0); assert.ok(p.state().atStart);
  p.step(1); p.step(1); assert.equal(p.state().index, 2);
  p.go(Infinity); assert.equal(p.state().index, 4);
  assert.deepEqual(events.map(e => e[1]), ["seek", "seek", "step", "step", "seek"]);
});

test("player plays at the chosen speed, stops at the end and restarts from the start", () => {
  const clock = fakeClock(), reasons = [];
  const p = T.createPlayer({count:3, onChange:(s, r) => reasons.push(r), schedule:clock.schedule, cancel:clock.cancel});
  p.setSpeed(3);
  p.play();
  assert.equal(clock.active[0].ms, T.SPEEDS[3].ms);
  assert.ok(p.state().playing);
  clock.tick(); clock.tick();
  assert.equal(p.state().index, 2);
  assert.equal(p.state().playing, false);
  assert.equal(clock.active.length, 0);
  assert.deepEqual(reasons, ["speed", "play", "tick", "end"]);
  p.toggle();
  assert.equal(p.state().index, 0);
  assert.ok(p.state().playing);
});

test("manual navigation and speed changes during playback behave predictably", () => {
  const clock = fakeClock(), p = T.createPlayer({count:10, schedule:clock.schedule, cancel:clock.cancel});
  p.play(); clock.tick();
  p.setSpeed(0);
  assert.ok(p.state().playing);
  assert.equal(clock.active.length, 1);
  assert.equal(clock.active[0].ms, T.SPEEDS[0].ms);
  p.step(1);
  assert.equal(p.state().playing, false);
  assert.equal(clock.active.length, 0);
  p.setSpeed(99); assert.equal(p.state().speed, T.SPEEDS.length - 1);
  const single = T.createPlayer({count:1, schedule:clock.schedule, cancel:clock.cancel});
  single.play(); assert.equal(single.state().playing, false);
});

test("phone layout lets the transport wrap and keeps content clear of the taller timeline", () => {
  const css = section("trace-style", "style");
  const phone = css.slice(css.indexOf("@media (max-width: 560px)"));
  assert.match(phone, /\.transport \{[^}]*flex-wrap: wrap/);
  assert.match(phone, /main \{ padding-bottom: 214px; \}/);
});

test("page UI never writes captured text through innerHTML", () => {
  const ui = section("trace-ui");
  assert.doesNotMatch(ui, /innerHTML|outerHTML\s*=|insertAdjacentHTML|document\.write/);
  for (const [file, tag] of [["trace-ui", "script"], ["trace-core", "script"], ["trace-style", "style"]]) {
    const text = section(file, tag).replaceAll('"http://www.w3.org/2000/svg"', "SVG_NS");
    assert.doesNotMatch(text, /<\/script|@font-face|@import|https?:\/\//i, file);
  }
});

test("stdout renders as terminal lines that keep diff marks, not as a JSON array", () => {
  const ui = section("trace-ui");
  assert.match(ui, /variable === "stdout" && Array\.isArray\(value\) && value\.every\(x => typeof x === "string"\)\) return output\(value, marks\)/);
  assert.match(ui, /"out-line", \.\.\.classify\(\[i\], marks\)/);
  const css = section("trace-style", "style");
  assert.match(css, /\.stdout \{[^}]*white-space: pre;[^}]*overflow: auto;/);
  assert.match(css, /\.stdout \.out-line\.mismatch \{/);
});

test("source lines grow with long code so the current-line highlight spans the scrolled width", () => {
  const css = section("trace-style", "style");
  assert.match(css, /\.line \{[^}]*width: max-content; min-width: 100%;/);
  assert.match(css, /\.code \{[^}]*overflow: auto;/);
});

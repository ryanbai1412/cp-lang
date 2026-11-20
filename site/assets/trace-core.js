// CP Lang trace core: validation, checkpoint alignment and state diffing.
// No DOM access; the page and test_trace.cjs load this same file.
"use strict";
const TraceCore = (() => {
  const own = (value, key) => value != null && Object.hasOwn(value, key);
  const record = value => value !== null && typeof value === "object" && !Array.isArray(value);
  const fail = message => { throw new Error(message); };
  const kinds = ["array", "grid", "dp", "graph", "tree", "scalar", "json"];
  function validate(trace) {
    // JSON.parse accepts 1e400 as Infinity and rounds unsafe integers. Refuse
    // these values rather than silently exporting a different state later.
    const pending = [trace];
    while (pending.length) {
      const value = pending.pop();
      if (typeof value === "number" && (!Number.isFinite(value) || Number.isInteger(value) && !Number.isSafeInteger(value))) fail("Non-finite numbers and integers outside the safe range must be encoded as strings");
      if (value !== null && typeof value === "object") for (const child of Object.values(value)) pending.push(child);
    }
    if (!record(trace) || trace.schemaVersion !== 1) fail("Expected a CP Lang trace with schemaVersion: 1");
    if (typeof trace.title !== "string" || !trace.title.trim()) fail("Trace title must be a non-empty string");
    if (trace.description !== undefined && typeof trace.description !== "string") fail("Description must be a string");
    if (!record(trace.source) || typeof trace.source.file !== "string" || typeof trace.source.code !== "string") fail("Trace source needs file and code strings");
    const lineCount = trace.source.code.split("\n").length;
    if (!record(trace.variables) || !Object.keys(trace.variables).length) fail("Trace needs at least one variable descriptor");
    for (const [id, v] of Object.entries(trace.variables)) {
      if (!record(v) || typeof v.name !== "string" || typeof v.type !== "string" || !kinds.includes(v.kind)) fail(`Invalid descriptor for ${id}`);
      for (const key of ["labels", "rowLabels", "colLabels"]) if (v[key] !== undefined && (!Array.isArray(v[key]) || v[key].some(x => !["number", "string"].includes(typeof x)))) fail(`${id}.${key} must be an array of labels`);
    }
    if (!record(trace.runs) || !own(trace.runs, "fast") || Object.keys(trace.runs).some(k => !["fast", "slow"].includes(k))) fail("Runs must contain fast and optionally slow");
    for (const [name, run] of Object.entries(trace.runs)) {
      if (!record(run) || typeof run.label !== "string" || !Array.isArray(run.steps) || !run.steps.length) fail(`${name} needs a label and at least one step`);
      if (run.steps.length > 100000) fail("Viewer supports at most 100,000 steps per run");
      const ids = new Set(), checkpoints = new Set();
      for (const [index, s] of run.steps.entries()) {
        const where = `${name} step ${index + 1}`;
        if (!record(s) || !Number.isInteger(s.id) || s.id < 0 || ids.has(s.id)) fail(`${where}: id must be a unique nonnegative integer`);
        ids.add(s.id);
        if (!Number.isInteger(s.line) || s.line < 1 || s.line > lineCount) fail(`${where}: source line is out of range`);
        if (typeof s.label !== "string" || typeof s.stage !== "string" || !record(s.values) || !record(s.locals)) fail(`${where}: expected label, stage, values and locals`);
        if (s.checkpoint !== undefined) {
          if (typeof s.checkpoint !== "string" || !s.checkpoint || checkpoints.has(s.checkpoint)) fail(`${where}: checkpoint must be a unique non-empty string`);
          checkpoints.add(s.checkpoint);
        }
        for (const [id, value] of Object.entries(s.values)) {
          if (!own(trace.variables, id)) fail(`${where}: unknown variable ${id}`);
          const v = trace.variables[id];
          if (v.kind === "array" && !Array.isArray(value)) fail(`${where}: ${id} must be an array`);
          if (["grid", "dp"].includes(v.kind) && (!Array.isArray(value) || value.some(row => !Array.isArray(row) || row.length !== (value[0]?.length ?? 0)))) fail(`${where}: ${id} must be a rectangular matrix`);
          if (["graph", "tree"].includes(v.kind)) validateGraph(value, v, `${where}: ${id}`);
          if (v.kind === "scalar" && value !== null && !["number", "string", "boolean"].includes(typeof value)) fail(`${where}: ${id} must be a scalar`);
        }
        if (s.focus !== undefined) {
          if (!record(s.focus) || !own(s.values, s.focus.variable) || !Array.isArray(s.focus.path) || s.focus.path.some(p => typeof p !== "string" && !(Number.isInteger(p) && p >= 0))) fail(`${where}: invalid focus path`);
          let value = s.values[s.focus.variable];
          for (const p of s.focus.path) { if (!own(value, p)) fail(`${where}: focus points outside the captured value`); value = value[p]; }
        }
      }
    }
    if (trace.compare !== undefined && (!Array.isArray(trace.compare) || !trace.compare.length || new Set(trace.compare).size !== trace.compare.length || trace.compare.some(id => !own(trace.variables, id)))) fail("compare must name one or more unique declared variables");
    let previous = -1;
    for (const pair of align(trace)) { if (pair.slowIndex <= previous) fail("Shared checkpoints must occur in the same order in both runs"); previous = pair.slowIndex; }
    return trace;
  }
  function validateGraph(value, descriptor, where) {
    if (!record(value) || !Array.isArray(value.nodes) || !Array.isArray(value.edges)) fail(`${where} needs nodes and edges`);
    const ids = new Set(), edgeIds = new Set(), parents = new Map(), children = new Map();
    for (const node of value.nodes) {
      if (!record(node) || !["string", "number"].includes(typeof node.id) || ids.has(String(node.id)) || typeof node.label !== "string") fail(`${where}: invalid or duplicate node`);
      ids.add(String(node.id));
      for (const key of ["x", "y"]) if (node[key] !== undefined && (!Number.isFinite(node[key]) || Math.abs(node[key]) > 100000)) fail(`${where}: invalid coordinate`);
      if (node.state !== undefined && !["visited", "frontier"].includes(node.state)) fail(`${where}: invalid node state`);
    }
    for (const e of value.edges) {
      if (!record(e) || !["string", "number"].includes(typeof e.id) || edgeIds.has(String(e.id)) || !ids.has(String(e.from)) || !ids.has(String(e.to))) fail(`${where}: invalid edge, id or endpoint`);
      edgeIds.add(String(e.id));
      if (e.weight !== undefined && typeof e.weight !== "number" && typeof e.weight !== "string") fail(`${where}: invalid edge weight`);
      const a = String(e.from), b = String(e.to);
      parents.set(b, (parents.get(b) || 0) + 1);
      if (!children.has(a)) children.set(a, []);
      children.get(a).push(b);
    }
    if (descriptor.kind === "tree" && value.nodes.length) {
      const root = descriptor.root ?? value.nodes[0].id;
      if (!ids.has(String(root)) || parents.has(String(root)) || [...ids].some(id => id !== String(root) && parents.get(id) !== 1)) fail(`${where}: tree must have one root and one parent per other node`);
      const seen = new Set(), queue = [String(root)];
      for (let i = 0; i < queue.length; i++) {
        const id = queue[i];
        if (seen.has(id)) fail(`${where}: tree contains a cycle`);
        seen.add(id); queue.push(...(children.get(id) || []));
      }
      if (seen.size !== ids.size) fail(`${where}: tree is disconnected`);
    }
  }
  function diff(a, b, path = [], aPresent = true, bPresent = true, result = []) {
    if (!aPresent || !bPresent) { result.push({path, fast:a, slow:b, fastPresent:aPresent, slowPresent:bPresent}); return result; }
    if (Object.is(a, b)) return result;
    if (a !== null && b !== null && typeof a === "object" && typeof b === "object" && Array.isArray(a) === Array.isArray(b)) {
      const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
      for (const key of keys) diff(a[key], b[key], [...path, Array.isArray(a) ? Number(key) : key], own(a,key), own(b,key), result);
    } else result.push({path, fast:a, slow:b, fastPresent:true, slowPresent:true});
    return result;
  }
  function align(trace) {
    if (!trace.runs.slow) return [];
    const slow = new Map(trace.runs.slow.steps.filter(s => s.checkpoint !== undefined).map(s => [s.checkpoint, s]));
    const indices = new Map(trace.runs.slow.steps.map((s,i) => [s.id,i]));
    return trace.runs.fast.steps.flatMap((step, fastIndex) => {
      const reference = slow.get(step.checkpoint);
      return reference ? [{fastIndex, slowIndex:indices.get(reference.id), checkpoint:step.checkpoint}] : [];
    });
  }
  function compare(trace, pair) {
    if (!pair) return [];
    const a = trace.runs.fast.steps[pair.fastIndex].values, b = trace.runs.slow.steps[pair.slowIndex].values;
    return (trace.compare ?? Object.keys(trace.variables)).flatMap(id => diff(a[id], b[id], [], own(a,id), own(b,id)).map(d => ({...d, variable:id})));
  }
  function analyze(trace) {
    const pairs = align(trace).map(pair => ({...pair, differences:compare(trace,pair)}));
    return {pairs, byFast:new Map(pairs.map(p => [p.fastIndex,p])), divergences:pairs.filter(p => p.differences.length)};
  }
  function pathName(name, path) { return name + path.map(p => typeof p === "number" ? `[${p}]` : /^[a-zA-Z_]\w*$/.test(p) ? `.${p}` : `[${JSON.stringify(p)}]`).join(""); }
  const containsPath = (parent,child) => parent.length <= child.length && parent.every((p,i) => p === child[i]);
  function selectDifference(differences, variable, path) {
    const candidates = differences.filter(d => d.variable === variable);
    if (path === null || path === undefined) return candidates[0];
    return candidates.find(d => d.path.length === path.length && containsPath(path,d.path))
      ?? candidates.find(d => containsPath(path,d.path));
  }

  // ---- Pure view-model helpers shared by the UI and the Node tests. ----
  const SPEEDS = [{label:"0.5×", ms:1200}, {label:"1×", ms:600}, {label:"2×", ms:300}, {label:"4×", ms:150}, {label:"8×", ms:75}];
  const clamp = (value, count) => Math.max(0, Math.min(count - 1, value));
  function format(value, present = true) {
    if (!present) return "out of scope";
    if (value === null) return "none";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  }
  function valueAt(values, variable, path) {
    let value = values?.[variable], present = own(values, variable);
    for (const key of path) { present = present && own(value, key); value = present ? value[key] : undefined; }
    return {value, present};
  }
  // The spec lane holds the last aligned reference at internal fast events.
  function referenceAt(analysis, fastIndex) {
    const exact = analysis.byFast.get(fastIndex);
    if (exact) return {pair:exact, exact:true};
    let held = null;
    for (const pair of analysis.pairs) { if (pair.fastIndex > fastIndex) break; held = pair; }
    return {pair:held, exact:false};
  }
  function adjacentDivergence(analysis, fastIndex, direction) {
    const list = analysis.divergences.map(p => p.fastIndex);
    return direction > 0 ? list.find(i => i > fastIndex) ?? null : list.filter(i => i < fastIndex).at(-1) ?? null;
  }
  // Leaf paths written by this step, relative to the previous snapshot in the same run.
  function changes(run, index, variables) {
    const step = run.steps[index], previous = run.steps[index - 1];
    if (!previous) return [];
    return variables.flatMap(id => diff(step.values[id], previous.values[id], [], own(step.values, id), own(previous.values, id))
      .map(d => ({variable:id, path:d.path, before:d.slow, after:d.fast, beforePresent:d.slowPresent, afterPresent:d.fastPresent})));
  }
  function summary(trace, analysis) {
    const first = analysis.divergences[0];
    if (!trace.runs.slow) return {state:"single", title:"Execution captured", detail:`${trace.runs.fast.steps.length} steps · scrub the timeline to replay every snapshot`};
    if (!analysis.pairs.length) return {state:"incomparable", title:"No shared checkpoints", detail:"The runs cannot be compared: add matching checkpoint names to both runs"};
    if (!first) return {state:"agree", title:"Fast and spec agree", detail:`All compared values match at ${analysis.pairs.length} shared checkpoints`};
    const d = first.differences[0];
    return {state:"diverge", title:`First divergence at step ${first.fastIndex + 1}`,
      detail:`${pathName(trace.variables[d.variable].name, d.path)}: fast ${format(d.fast, d.fastPresent)} ≠ spec ${format(d.slow, d.slowPresent)} · checkpoint ${first.checkpoint}`,
      first, difference:d};
  }
  function keyAction(event) {
    if (event.altKey || event.ctrlKey || event.metaKey) return null;
    if (event.key === "ArrowRight") return event.shiftKey ? "next-divergence" : "next";
    if (event.key === "ArrowLeft") return event.shiftKey ? "previous-divergence" : "previous";
    if (event.key === "Home") return "first";
    if (event.key === "End") return "last";
    if (event.code === "Space" || event.key === " ") return "toggle";
    if (event.key === "]") return "next-divergence";
    if (event.key === "[") return "previous-divergence";
    return null;
  }
  // Tidy rooted layout: leaves take consecutive columns, parents center over children.
  function layout(value, descriptor) {
    const positions = new Map();
    if (!value.nodes.length) return positions;
    if (descriptor.kind === "tree") {
      const children = new Map(), root = String(descriptor.root ?? value.nodes[0].id);
      for (const e of value.edges) { const key = String(e.from); if (!children.has(key)) children.set(key, []); children.get(key).push(String(e.to)); }
      let column = 0;
      const place = (id, depth) => {
        const kids = children.get(id) || [];
        const xs = kids.map(kid => place(kid, depth + 1));
        const x = xs.length ? (xs[0] + xs.at(-1)) / 2 : column++;
        positions.set(id, {x, y:depth});
        return x;
      };
      // Iterative guard is unnecessary: validate() rejects cycles and caps vertices.
      place(root, 0);
      for (const [id, p] of positions) positions.set(id, {x:40 + p.x * 72, y:40 + p.y * 80});
      return positions;
    }
    const n = value.nodes.length, radius = Math.max(90, n * 16);
    value.nodes.forEach((node, i) => positions.set(String(node.id), {
      x: Number.isFinite(node.x) ? node.x : radius + radius * Math.cos(2 * Math.PI * i / n - Math.PI / 2),
      y: Number.isFinite(node.y) ? node.y : radius + radius * Math.sin(2 * Math.PI * i / n - Math.PI / 2)}));
    return positions;
  }
  // Padded SVG viewBox around laid-out vertices; null when there is nothing to draw.
  function viewBox(positions, pad = 32) {
    const points = [...positions.values()];
    if (!points.length) return null;
    const xs = points.map(p => p.x), ys = points.map(p => p.y);
    const x = Math.min(...xs) - pad, y = Math.min(...ys) - pad;
    return {x, y, w:Math.max(...xs) - x + pad, h:Math.max(...ys) - y + pad};
  }

  function legend(kind, comparing) {
    const items = {
      dp:[["changed","Written this step"],["focus","Current cell"],["filled","Non-zero"]],
      grid:[["wall","Wall"],["endpoint","Start / goal"],["focus","Current cell"],["changed","Written this step"]],
      array:[["changed","Written this step"],["focus","Current index"]],
      graph:[["node","Unvisited"],["frontier","Frontier"],["visited","Visited"],["focus","Current"]],
      tree:[["node","Pending"],["visited","Done"],["focus","Current"]],
      scalar:[["changed","Written this step"]],
      json:[],
    }[kind] || [];
    return comparing ? [...items, ["mismatch","Fast ≠ spec"]] : items;
  }
  // Transport state machine; the scheduler is injected so playback is testable.
  function createPlayer({count, onChange, schedule = setInterval, cancel = clearInterval}) {
    let index = 0, speed = 1, timer = null;
    const state = () => ({index, speed, playing:timer !== null, atStart:index === 0, atEnd:index === count - 1});
    const emit = reason => onChange?.(state(), reason);
    const halt = () => { if (timer !== null) cancel(timer); timer = null; };
    const player = {
      state,
      setCount(next) { halt(); count = next; index = 0; emit("load"); },
      go(next, reason = "seek") { halt(); index = clamp(next, count); emit(reason); },
      step(delta) { player.go(index + delta, "step"); },
      stop() { if (timer !== null) { halt(); emit("pause"); } },
      play() {
        if (timer !== null || count < 2) return;
        if (index === count - 1) index = 0;
        timer = schedule(() => {
          index = clamp(index + 1, count);
          if (index === count - 1) halt();
          emit(timer === null ? "end" : "tick");
        }, SPEEDS[speed].ms);
        emit("play");
      },
      toggle() { timer !== null ? player.stop() : player.play(); },
      setSpeed(next) {
        speed = Math.max(0, Math.min(SPEEDS.length - 1, next));
        if (timer !== null) { halt(); player.play(); } else emit("speed");
      },
    };
    return player;
  }
  return {validate, diff, align, compare, analyze, pathName, own, containsPath, selectDifference,
    SPEEDS, clamp, format, valueAt, referenceAt, adjacentDivergence, changes, summary, keyAction, layout, viewBox, legend, createPlayer};
})();
if (typeof module !== "undefined") module.exports = TraceCore;

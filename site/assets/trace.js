// CP Lang trace debugger UI. Every captured string reaches the page through
// textContent or setAttribute, never as parsed markup, so traces cannot inject HTML.
(() => {
  "use strict";
  const T = TraceCore, $ = id => document.getElementById(id);
  const SVG = "http://www.w3.org/2000/svg", MAX_IMPORT = 50 * 1024 * 1024;
  const LIMIT = {matrix:100, array:500, graph:500, log:500, marks:1000};

  function el(tag, props = {}, ...kids) {
    const node = props.svg ? document.createElementNS(SVG, tag) : document.createElement(tag);
    for (const [key, value] of Object.entries(props)) {
      if (key === "svg" || value === undefined || value === null || value === false) continue;
      if (key === "text") node.textContent = value;
      else if (key === "class") node.setAttribute("class", value);
      else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
      else node.setAttribute(key, value === true ? "" : String(value));
    }
    for (const kid of kids.flat()) if (kid !== null && kid !== undefined && kid !== false) node.append(kid);
    return node;
  }
  const svg = (tag, props = {}, ...kids) => el(tag, {...props, svg:true}, ...kids);
  const pad = (n, width) => String(n).padStart(width, "0");
  const key = path => JSON.stringify(path);
  const display = v => v === null ? "·" : Array.isArray(v) ? `(${v.map(display).join(", ")})` : typeof v === "object" ? JSON.stringify(v) : String(v);
  const hashParam = name => new URLSearchParams(location.hash.slice(1)).get(name);

  let samples = [], sampleIndex = -1, trace, analysis, variable, comparing = false, selected = null, tab = "state", search = null;
  const player = T.createPlayer({count:1, onChange:(state, reason) => {
    if (reason === "step" || reason === "seek") selected = null;
    render();
    if (reason !== "tick") announce();
  }});

  // ---------- Loading ----------
  function load(next, index = -1) {
    T.validate(next);
    trace = next; sampleIndex = index; analysis = T.analyze(trace); search = null; selected = null;
    comparing = analysis.pairs.length > 0;
    const first = analysis.divergences[0];
    variable = first?.differences[0].variable ?? trace.runs.fast.steps[0].focus?.variable ?? Object.keys(trace.variables)[0];
    player.setCount(trace.runs.fast.steps.length);
    renderStatic();
    if (first) jumpToFirst();
  }
  function tryLoad(next, index = -1) {
    try { load(next, index); return true; } catch (error) { toast(`Could not open trace: ${error.message}`, true); return false; }
  }
  function parseFile(text) {
    if (/^\s*</.test(text)) {
      const doc = new DOMParser().parseFromString(text, "text/html");
      const data = JSON.parse(doc.getElementById("trace-data")?.textContent || "null");
      const embedded = JSON.parse(doc.getElementById("trace-samples")?.textContent || "[]");
      const found = data ?? embedded[0];
      if (!found) throw new Error("the HTML file contains no trace");
      return found;
    }
    return JSON.parse(text);
  }
  async function openFile(file) {
    if (!file) return;
    if (file.size > MAX_IMPORT) return toast("Trace files are limited to 50 MiB", true);
    try { if (tryLoad(parseFile(await file.text()))) toast(`Opened ${file.name}`); }
    catch (error) { toast(`Could not read ${file.name}: ${error.message}`, true); }
  }

  // ---------- Derived state ----------
  function view() {
    const i = player.state().index, fast = trace.runs.fast, step = fast.steps[i];
    const ref = comparing ? T.referenceAt(analysis, i) : {pair:null, exact:false};
    const slowIndex = ref.pair?.slowIndex ?? null;
    const differences = ref.exact ? ref.pair.differences : [];
    return {i, step, ref, slowIndex, slow:slowIndex === null ? null : trace.runs.slow.steps[slowIndex], differences};
  }
  const nameOf = id => trace.variables[id].name;

  // ---------- Rendering ----------
  function renderStatic() { keepFocus(renderStaticAll); }
  function renderStaticAll() {
    $("title").textContent = trace.title;
    $("desc").textContent = trace.description ?? "";
    document.title = `${trace.title} · Trace debugger · CP Lang 0.2.2`;
    const slow = trace.runs.slow;
    $("stats").replaceChildren(...[
      ["Fast steps", trace.runs.fast.steps.length],
      ...(slow ? [["Spec steps", slow.steps.length], ["Checkpoints", analysis.pairs.length], ["Divergent", analysis.divergences.length]] : []),
    ].map(([label, value]) => el("div", {}, el("dt", {text:label}), el("dd", {text:value}))));
    $("samples").replaceChildren(...(samples.length ? samples : [trace]).map((sample, index) => el("li", {}, el("button", {
      class:"item", type:"button", "data-focus":`sample:${index}`, "aria-current":String(samples.length ? index === sampleIndex : true),
      onclick:() => samples.length && index !== sampleIndex && tryLoad(samples[index], index),
    }, el("span", {class:"item-name", text:sample.title}), el("span", {class:"item-meta", text:Object.values(sample.variables)[0]?.kind ?? ""})))));
    if (sampleIndex < 0 && samples.length) $("samples").prepend(el("li", {}, el("button", {class:"item", type:"button", "aria-current":"true"}, el("span", {class:"item-name", text:trace.title}), el("span", {class:"item-meta", text:"opened"}))));
    $("modes").hidden = $("diff-panel").hidden = !slow;
    $("mode-compare").disabled = !analysis.pairs.length;
    renderStatus(); renderMarks(); renderSpeeds(); render();
  }

  function renderStatus() {
    const s = T.summary(trace, analysis), box = $("status");
    box.dataset.state = s.state;
    const actions = [];
    if (s.first) {
      const total = analysis.divergences.length;
      actions.push(
        el("button", {class:"btn icon", type:"button", "aria-label":"Previous divergence", title:"Previous divergence ([)", onclick:() => divergence(-1)}, chevron("M10 3 5 8l5 5")),
        el("button", {class:"btn icon", type:"button", "aria-label":"Next divergence", title:`Next divergence (]) · ${total} divergent checkpoint${total === 1 ? "" : "s"}`, onclick:() => divergence(1)}, chevron("m6 3 5 5-5 5")),
        el("button", {class:"btn primary", type:"button", onclick:jumpToFirst, text:"Jump to first divergence"}));
    }
    box.replaceChildren(el("span", {class:"status-icon", "aria-hidden":"true"}),
      el("div", {class:"status-text"}, el("div", {class:"status-title", text:s.title}), el("div", {class:"status-detail", text:s.detail})),
      el("div", {class:"status-actions"}, actions));
  }
  const chevron = d => svg("svg", {viewBox:"0 0 16 16", fill:"none", stroke:"currentColor", "stroke-width":"1.6", "aria-hidden":"true"}, svg("path", {d}));

  function jumpToFirst() {
    const first = analysis.divergences[0];
    if (!first) return;
    comparing = true; setTab("state");
    player.go(first.fastIndex);
    variable = first.differences[0].variable; selected = first.differences[0].path; render();
  }
  function divergence(direction) {
    const target = T.adjacentDivergence(analysis, player.state().index, direction);
    if (target === null) return toast(direction > 0 ? "No later divergence" : "No earlier divergence");
    comparing = true; player.go(target);
    const d = analysis.byFast.get(target).differences[0];
    variable = d.variable; selected = d.path; render();
  }

  // Re-rendering replaces nodes; put keyboard focus back on the equivalent new node.
  function focusSelector(node) {
    if (!(node instanceof Element)) return null;
    if (node.dataset.focus) return `[data-focus="${CSS.escape(node.dataset.focus)}"]`;
    const lane = node.closest("[data-lane]");
    return lane && node.dataset.path ? `[data-lane="${lane.dataset.lane}"] [data-path="${CSS.escape(node.dataset.path)}"]` : null;
  }
  function keepFocus(update) {
    const before = document.activeElement, selector = focusSelector(before);
    update();
    if (selector && !before.isConnected) document.querySelector(selector)?.focus({preventScroll:true});
  }

  function render() { keepFocus(renderAll); }
  function renderAll() {
    if (!trace) return;
    const v = view();
    $("mode-single").setAttribute("aria-pressed", String(!comparing));
    $("mode-compare").setAttribute("aria-pressed", String(comparing));
    renderVars(v); renderInspector(v); renderDiffs(v); renderChanges(v); renderSource(v); renderLocals(v); renderTimeline(v);
    if (tab === "log") renderLog(v);
  }

  function renderVars(v) {
    $("vars").replaceChildren(...Object.entries(trace.variables).map(([id, d]) => {
      const count = v.differences.filter(x => x.variable === id).length;
      return el("li", {}, el("button", {class:"item", type:"button", "data-focus":`var:${id}`, "aria-pressed":String(id === variable), title:d.type ?? d.kind,
        onclick:() => { variable = id; selected = null; render(); }},
        el("span", {class:"item-name mono", text:d.name}), count ? el("span", {class:"count", text:count, title:`${count} value${count === 1 ? "" : "s"} differ from spec`}) : el("span", {class:"item-meta", text:d.kind})));
    }));
  }

  function renderInspector(v) {
    const d = trace.variables[variable], lanes = [lane("fast", v)];
    if (comparing) lanes.push(lane("slow", v));
    $("var-name").textContent = d.name;
    let meta = d.type ?? d.kind;
    if (selected) {
      const fast = T.valueAt(v.step.values, variable, selected);
      meta = `${T.pathName(d.name, selected)} = ${T.format(fast.value, fast.present)}`;
      if (v.slow) { const spec = T.valueAt(v.slow.values, variable, selected); meta += ` · spec ${T.format(spec.value, spec.present)}`; }
    }
    $("var-type").textContent = meta;
    $("lanes").className = comparing ? "lanes two" : "lanes";
    $("lanes").replaceChildren(...lanes);
    const items = T.legend(d.kind, comparing).map(([cls, label]) => el("span", {}, el("i", {class:`sw ${cls}`, "aria-hidden":"true"}), label));
    if (d.rowDomain || d.colDomain) items.push(el("span", {class:"axis", text:[d.rowDomain && `rows ${d.rowDomain}`, d.colDomain && `cols ${d.colDomain}`].filter(Boolean).join(" · ")}));
    $("legend").replaceChildren(...items);
    $("legend").hidden = !items.length;
  }

  function lane(which, v) {
    const isFast = which === "fast", run = isFast ? trace.runs.fast : trace.runs.slow;
    const index = isFast ? v.i : v.slowIndex, step = isFast ? v.step : v.slow;
    const head = el("div", {class:"lane-head"},
      el("span", {class:`badge ${isFast ? "fast" : "spec"}`, text:isFast ? "FAST" : "SPEC"}),
      el("span", {class:"lane-name", text:run.label}),
      el("span", {class:"lane-meta", text:step ? `step ${index + 1} · L${step.line}${step.checkpoint !== undefined ? ` · ${step.checkpoint}` : ""}` : "not reached"}));
    const body = el("div", {class:"viz"});
    let note = null;
    if (!step) body.append(el("p", {class:"empty", text:"No spec state yet: the fast run has not reached a shared checkpoint"}));
    else {
      const marks = laneMarks(isFast, run, index, step, v);
      body.append(visualize(step, marks, isFast ? "fast" : "spec"));
      if (!isFast && !v.ref.exact) note = el("p", {class:"lane-note", text:`Holding the last aligned spec state (checkpoint ${v.ref.pair.checkpoint}); this fast step has no spec counterpart, so nothing is compared.`});
    }
    return el("section", {class:"lane", "data-lane":which, "aria-label":isFast ? "Fast run" : "Spec run"}, head, body, note);
  }

  function laneMarks(isFast, run, index, step, v) {
    const changed = new Map(T.changes(run, index, [variable]).map(c => [key(c.path), c]));
    const differences = v.differences.filter(d => d.variable === variable);
    const focus = step.focus?.variable === variable ? step.focus.path : null;
    return {changed, differences, focus, isFast, other:isFast ? v.slow : v.step};
  }
  function classify(path, marks) {
    const cls = [];
    if ([...marks.changed.values()].some(c => T.containsPath(path, c.path))) cls.push("changed");
    if (marks.focus && key(marks.focus) === key(path)) cls.push("focus");
    if (marks.differences.some(d => T.containsPath(path, d.path) || T.containsPath(d.path, path))) cls.push(marks.isFast ? "mismatch" : "expected");
    if (selected && key(selected) === key(path)) cls.push("selected");
    return cls;
  }
  function tip(path, value, marks, cls) {
    const name = T.pathName(nameOf(variable), path), lines = [`${name} = ${display(value)}`];
    if (cls.includes("mismatch") || cls.includes("expected")) {
      const other = T.valueAt(marks.other?.values, variable, path);
      lines.push(`${marks.isFast ? "spec" : "fast"} = ${T.format(other.value, other.present)}`);
    }
    const write = marks.changed.get(key(path));
    if (write) lines.push(`was ${T.format(write.before, write.beforePresent)}`);
    if (cls.includes("focus")) lines.push("current write");
    return lines.join("\n");
  }
  function cell(tag, path, value, text, marks, extra = []) {
    const cls = classify(path, marks), label = tip(path, value, marks, cls);
    return el(tag, {class:["cell", ...extra, ...cls].join(" "), "data-path":key(path), "data-tip":label, "aria-label":label.replace(/\n/g, ", "), "aria-pressed":String(cls.includes("selected")), tabindex:"0", role:"button", text});
  }

  function visualize(step, marks, lane) {
    const d = trace.variables[variable];
    if (!T.own(step.values, variable)) return el("p", {class:"empty", text:"Not in scope at this step"});
    const value = step.values[variable];
    if ((d.kind === "dp" || d.kind === "grid") && Array.isArray(value)) return matrix(value, d, marks);
    if (d.kind === "array" && Array.isArray(value)) return array(value, d, marks);
    if ((d.kind === "graph" || d.kind === "tree") && value && Array.isArray(value.nodes)) return graph(value, d, marks, lane);
    if (d.kind === "scalar" || value === null || typeof value !== "object") {
      const cls = classify([], marks);
      return el("div", {class:`scalar cell ${cls.join(" ")}`, "data-path":"[]", "data-tip":tip([], value, marks, cls), "aria-pressed":String(cls.includes("selected")), tabindex:"0", role:"button", text:display(value)});
    }
    return el("pre", {class:"json", text:JSON.stringify(value, null, 2)});
  }

  function matrix(value, d, marks) {
    const rows = value.slice(0, LIMIT.matrix), width = Math.min(LIMIT.matrix, Math.max(0, ...rows.map(r => r.length)));
    const focus = marks.focus ?? [];
    const head = el("tr", {}, el("th"), Array.from({length:width}, (_, c) => el("th", {scope:"col", class:focus[1] === c ? "on" : null, text:d.colLabels?.[c] ?? c})));
    const body = rows.map((row, r) => el("tr", {}, el("th", {scope:"row", class:`rh${focus[0] === r ? " on" : ""}`, text:d.rowLabels?.[r] ?? r}),
      row.slice(0, width).map((x, c) => {
        const extra = d.kind === "grid" ? (x === "#" ? ["wall"] : /^[A-Z]$/.test(String(x)) ? ["endpoint"] : x === "." ? ["zero"] : [])
          : x === null || x === 0 ? ["zero"] : ["filled"];
        return cell("td", [r, c], x, d.kind === "grid" && x === "." ? "" : display(x), marks, extra);
      })));
    const table = el("table", {class:"matrix", "aria-label":`${d.name}: ${value.length} × ${value[0]?.length ?? 0}`}, el("thead", {}, head), el("tbody", {}, body));
    const clipped = value.length > LIMIT.matrix || value.some(r => r.length > LIMIT.matrix);
    return clipped ? el("div", {}, table, el("p", {class:"lane-note", text:`Showing the first ${LIMIT.matrix} × ${LIMIT.matrix} cells`})) : table;
  }

  function array(value, d, marks) {
    const items = value.slice(0, LIMIT.array);
    const numeric = items.length > 0 && items.length <= 64 && items.every(x => typeof x === "number");
    const max = numeric ? Math.max(1, ...items.map(Math.abs)) : 1;
    const parts = [];
    if (numeric) parts.push(el("div", {class:"bars", "aria-hidden":"true"}, items.map((x, i) => {
      const cls = classify([i], marks).filter(c => c === "changed" || c === "mismatch").join(" ");
      return el("i", {class:cls || null, style:`height:${Math.round(Math.abs(x) / max * 100)}%`});
    })));
    if (!items.length) parts.push(el("p", {class:"empty", text:"Empty"}));
    parts.push(el("div", {class:"array"}, items.map((x, i) => el("div", {class:"slot"},
      el("span", {class:"idx", text:d.labels?.[i] ?? i}), cell("div", [i], x, display(x), marks, x === null || x === 0 ? ["zero"] : [])))));
    if (value.length > LIMIT.array) parts.push(el("p", {class:"lane-note", text:`Showing ${LIMIT.array} of ${value.length} elements`}));
    return el("div", {"aria-label":`${d.name}: ${value.length} elements`}, parts);
  }

  function graph(value, d, marks, lane) {
    if (value.nodes.length > LIMIT.graph) return el("p", {class:"empty", text:`${d.kind === "tree" ? "Tree" : "Graph"} has ${value.nodes.length} vertices, more than the ${LIMIT.graph} this view draws. Save JSON to inspect the full state.`});
    const pos = T.layout(value, d);
    const box = T.viewBox(pos);
    if (!box) return el("p", {class:"empty", text:`${d.kind === "tree" ? "Empty tree" : "Empty graph"}: no vertices at this step`});
    const {x:minX, y:minY, w, h} = box, arrow = `arrow-${lane}`;
    const edges = value.edges.map((e, k) => {
      const a = pos.get(String(e.from)), b = pos.get(String(e.to));
      if (!a || !b) return null;
      const path = ["edges", k], cls = classify(path, marks);
      const len = Math.hypot(b.x - a.x, b.y - a.y) || 1, ux = (b.x - a.x) / len, uy = (b.y - a.y) / len;
      const label = `${nodeLabel(value, e.from)} → ${nodeLabel(value, e.to)}${e.weight !== undefined ? ` · weight ${display(e.weight)}` : ""}`;
      return svg("g", {"data-path":key(path), "data-tip":label},
        svg("line", {class:`edge${cls.includes("focus") ? " active" : ""}${cls.includes("mismatch") ? " mismatch" : ""}`,
          x1:a.x + ux * 17, y1:a.y + uy * 17, x2:b.x - ux * 19, y2:b.y - uy * 19, "marker-end":d.directed ? `url(#${arrow})` : null}),
        e.weight !== undefined ? svg("text", {class:"weight", x:(a.x + b.x) / 2, y:(a.y + b.y) / 2 - 4, "text-anchor":"middle", text:display(e.weight)}) : null);
    });
    const circles = value.nodes.map((n, k) => {
      const p = pos.get(String(n.id));
      if (!p) return null;
      const path = ["nodes", k], cls = classify(path, marks).filter(c => c !== "changed");
      if (n.state) cls.push(n.state);
      const lines = [`${n.label ?? n.id}${n.state ? ` · ${n.state}` : " · unvisited"}`];
      if (cls.includes("mismatch") || cls.includes("expected")) {
        const other = T.valueAt(marks.other?.values, variable, path).value;
        lines.push(`${marks.isFast ? "spec" : "fast"}: ${other?.state ?? "unvisited"}`);
      }
      return svg("g", {class:`node ${cls.join(" ")}`, transform:`translate(${p.x} ${p.y})`, "data-path":key(path), "data-tip":lines.join("\n"), "aria-label":lines.join(", "), "aria-pressed":String(cls.includes("selected")), tabindex:"0", role:"button"},
        svg("circle", {r:16}), svg("text", {text:n.label ?? n.id}));
    });
    return svg("svg", {class:"graph", viewBox:`${minX} ${minY} ${w} ${h}`, width:Math.min(w, 640), role:"img", "aria-label":`${d.name}: ${value.nodes.length} vertices, ${value.edges.length} edges`},
      svg("defs", {}, svg("marker", {id:arrow, viewBox:"0 0 8 8", refX:"7", refY:"4", markerWidth:"7", markerHeight:"7", orient:"auto-start-reverse"}, svg("path", {d:"M0 0 8 4 0 8z", fill:"currentColor", class:"edge-tip"}))),
      edges, circles);
  }
  const nodeLabel = (value, id) => { const n = value.nodes.find(x => String(x.id) === String(id)); return n?.label ?? id; };

  function renderDiffs(v) {
    const box = $("diffs");
    if (!comparing) { $("diff-meta").textContent = ""; return box.replaceChildren(el("p", {class:"empty", text:trace.runs.slow ? "Switch to Fast vs spec to compare runs" : "This trace has no spec run"})); }
    if (!v.ref.exact) { $("diff-meta").textContent = "no checkpoint"; return box.replaceChildren(el("p", {class:"empty", text:"This fast step is internal: compare at the next shared checkpoint"})); }
    const list = v.differences;
    $("diff-meta").textContent = `${v.ref.pair.checkpoint} · ${list.length ? `${list.length} differ` : "all equal"}`;
    if (!list.length) return box.replaceChildren(el("p", {class:"empty", text:"Every compared value matches the spec here"}));
    box.replaceChildren(el("ul", {class:"rows"}, el("li", {}, el("div", {class:"row head-row"}, el("span", {text:"Path"}), el("span", {text:"Fast"}), el("span", {text:"Spec"}))),
      list.slice(0, 200).map(d => el("li", {}, el("button", {class:"row", type:"button", "data-focus":`diff:${d.variable}:${key(d.path)}`, "aria-pressed":String(d.variable === variable && selected && key(selected) === key(d.path)),
        onclick:() => { variable = d.variable; selected = d.path; render(); }},
        el("span", {text:T.pathName(nameOf(d.variable), d.path)}), el("span", {class:"bad", text:T.format(d.fast, d.fastPresent)}), el("span", {class:"exp", text:T.format(d.slow, d.slowPresent)}))))));
    if (list.length > 200) box.append(el("p", {class:"lane-note", text:`Showing first 200 of ${list.length}; JSON contains full state`}));
  }

  function renderChanges(v) {
    const list = T.changes(trace.runs.fast, v.i, Object.keys(trace.variables));
    $("change-meta").textContent = v.i === 0 ? "initial state" : `${list.length} write${list.length === 1 ? "" : "s"}`;
    if (!list.length) return $("changes").replaceChildren(el("p", {class:"empty", text:v.i === 0 ? "First snapshot: nothing to compare against" : "No captured value changed"}));
    $("changes").replaceChildren(el("ul", {class:"rows"}, el("li", {}, el("div", {class:"row head-row"}, el("span", {text:"Path"}), el("span", {text:"Before"}), el("span", {text:"After"}))),
      list.slice(0, 200).map(c => el("li", {}, el("button", {class:"row", type:"button", "data-focus":`change:${c.variable}:${key(c.path)}`, "aria-pressed":String(c.variable === variable && selected && key(selected) === key(c.path)),
        onclick:() => { variable = c.variable; selected = c.path; render(); }},
        el("span", {text:T.pathName(nameOf(c.variable), c.path)}), el("span", {class:"old", text:T.format(c.before, c.beforePresent)}), el("span", {text:T.format(c.after, c.afterPresent)}))))));
    if (list.length > 200) $("changes").append(el("p", {class:"lane-note", text:`Showing first 200 of ${list.length}; JSON contains full state`}));
  }

  const TOKENS = /(\/\/[^\n]*)|("(?:\\.|[^"\\])*")|(@!?\w+)|(\b\d[\d_]*\b)|(\b(?:let|var|const|fn|struct|enum|domain|type|use|if|elif|else|for|while|in|match|return|break|continue|and|or|not|assert|mut|self|true|false|none|trait|impl|lends|yield|unchecked)\b)/g;
  function highlight(line) {
    const out = []; let last = 0;
    for (const m of line.matchAll(TOKENS)) {
      if (m.index > last) out.push(line.slice(last, m.index));
      const cls = m[1] ? "tk-c" : m[2] ? "tk-s" : m[3] ? "tk-a" : m[4] ? "tk-n" : "tk-k";
      out.push(el("span", {class:cls, text:m[0]})); last = m.index + m[0].length;
    }
    if (last < line.length) out.push(line.slice(last));
    return out;
  }
  let sourceKey = null;
  function renderSource(v) {
    const box = $("source"), lines = trace.source.code.split("\n");
    if (sourceKey !== trace) {
      sourceKey = trace;
      box.replaceChildren(...lines.map((line, i) => el("div", {class:"line", "data-line":i + 1}, el("span", {class:"n", text:i + 1}), el("span", {}, highlight(line)))));
    }
    box.querySelectorAll(".line.on").forEach(n => n.classList.remove("on"));
    const active = box.children[v.step.line - 1];
    if (active) {
      active.classList.add("on");
      const top = active.offsetTop - box.clientHeight / 2;
      if (active.offsetTop < box.scrollTop || active.offsetTop > box.scrollTop + box.clientHeight - 20) box.scrollTop = Math.max(0, top);
    }
    $("src-meta").textContent = `${trace.source.file ?? "source"} · line ${v.step.line}`;
  }
  function renderLocals(v) {
    const entries = Object.entries(v.step.locals ?? {});
    $("locals-meta").textContent = v.step.stage ?? "";
    $("locals").replaceChildren(...(entries.length ? entries.flatMap(([k, x]) => [el("dt", {text:k}), el("dd", {text:display(x)})]) : [el("dt", {text:"—"}), el("dd", {text:"no locals captured"})]));
  }

  function renderMarks() {
    const n = trace.runs.fast.steps.length, at = i => `${n > 1 ? i / (n - 1) * 100 : 0}%`;
    const pairs = analysis.pairs.length <= 400 ? analysis.pairs : analysis.divergences;
    $("marks").replaceChildren(...pairs.slice(0, LIMIT.marks).map(p => el("i", {class:p.differences.length ? "x" : null, style:`left:${at(p.fastIndex)}`})));
  }
  function renderSpeeds() {
    $("speeds").replaceChildren(...T.SPEEDS.map((s, i) => el("button", {type:"button", "aria-pressed":String(player.state().speed === i), "aria-label":`Speed ${s.label}`, text:s.label, onclick:() => player.setSpeed(i)})));
  }
  function renderTimeline(v) {
    const s = player.state(), n = trace.runs.fast.steps.length, width = String(n).length;
    const scrub = $("scrub");
    scrub.max = n - 1; scrub.value = v.i;
    scrub.style.setProperty("--p", `${n > 1 ? v.i / (n - 1) * 100 : 0}%`);
    scrub.setAttribute("aria-valuetext", `Step ${v.i + 1} of ${n}: ${v.step.label}`);
    $("tl-kind").textContent = (v.step.stage ?? "Step").toUpperCase();
    $("tl-label").textContent = v.step.label + (v.differences.length ? ` · ${v.differences.length} differ from spec` : "");
    $("tl-pos").textContent = `${pad(v.i + 1, width)} / ${n}`;
    for (const id of ["first", "prev"]) $(id).disabled = s.atStart;
    for (const id of ["next", "last"]) $(id).disabled = s.atEnd;
    $("play").disabled = n < 2;
    $("play").setAttribute("aria-label", s.playing ? "Pause" : "Play");
    $("play-icon").setAttribute("d", s.playing ? "M4.5 3h2.5v10H4.5zM9 3h2.5v10H9z" : "M5 3v10l8-5z");
    [...$("speeds").children].forEach((b, i) => b.setAttribute("aria-pressed", String(s.speed === i)));
  }

  function renderLog(v) {
    const steps = trace.runs.fast.steps;
    search ??= steps.map(s => [s.label, `L${s.line}`, s.stage, s.checkpoint, JSON.stringify(s.values), JSON.stringify(s.locals)].join(" ").toLowerCase());
    const q = $("filter").value.trim().toLowerCase();
    const matches = []; for (let i = 0; i < steps.length; i++) if (!q || search[i].includes(q)) matches.push(i);
    const at = Math.max(0, matches.findIndex(i => i >= v.i)), start = Math.max(0, Math.min(at - LIMIT.log / 2, matches.length - LIMIT.log));
    const shown = matches.slice(start, start + LIMIT.log);
    $("log-meta").textContent = `${matches.length} of ${steps.length} events${matches.length > shown.length ? ` · showing ${shown.length}` : ""}`;
    $("log").replaceChildren(...shown.map(i => {
      const s = steps[i], diverged = analysis.byFast.get(i)?.differences.length;
      return el("li", {}, el("button", {class:"ev", type:"button", "data-focus":`log:${i}`, "aria-current":String(i === v.i), onclick:() => { player.go(i); }},
        el("span", {text:pad(i + 1, String(steps.length).length)}), el("span", {text:`L${s.line}`}), el("span", {class:"lbl", text:s.label}),
        el("span", {class:diverged ? "x" : null, text:diverged ? `≠ ${s.checkpoint}` : s.checkpoint ?? ""})));
    }));
    $("log").querySelector('[aria-current="true"]')?.scrollIntoView({block:"nearest"});
  }

  // ---------- Chrome ----------
  function setTab(next) {
    tab = next;
    for (const [id, panel] of [["tab-state", "view-state"], ["tab-log", "view-log"]]) {
      const on = id === `tab-${next}`;
      $(id).setAttribute("aria-selected", String(on)); $(id).tabIndex = on ? 0 : -1; $(panel).hidden = !on;
    }
    if (trace) render();
  }
  let toastTimer;
  function toast(text, bad = false) {
    const box = $("toast"); box.textContent = text; box.className = bad ? "toast bad" : "toast"; box.hidden = false;
    clearTimeout(toastTimer); toastTimer = setTimeout(() => { box.hidden = true; }, 3500);
  }
  let announceTimer;
  function announce() {
    clearTimeout(announceTimer);
    announceTimer = setTimeout(() => {
      const v = view(), n = trace.runs.fast.steps.length;
      $("announce").textContent = `Step ${v.i + 1} of ${n}, ${v.step.label}${v.differences.length ? `, ${v.differences.length} values differ from spec` : ""}`;
    }, 150);
  }
  const safeJSON = value => JSON.stringify(value).replace(/</g, "\\u003c").replace(/\u2028/g, "\\u2028").replace(/\u2029/g, "\\u2029");
  const slugify = text => text.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "trace";
  function download(name, type, text) {
    const url = URL.createObjectURL(new Blob([text], {type}));
    el("a", {href:url, download:name}).click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  function exportHTML() {
    const root = document.documentElement.cloneNode(true);
    for (const id of ["stats", "status", "samples", "vars", "lanes", "legend", "diffs", "changes", "source", "locals", "log", "marks", "speeds", "announce"]) root.querySelector(`#${id}`).replaceChildren();
    root.querySelector("#trace-data").textContent = safeJSON(trace);
    root.querySelector("#trace-samples").textContent = "[]";
    download(`${slugify(trace.title)}.trace.html`, "text/html", `<!doctype html>\n${root.outerHTML}`);
  }

  function bindTip() {
    const box = $("tip");
    const show = target => {
      const holder = target?.closest?.("[data-tip]");
      if (!holder) return void (box.hidden = true);
      box.textContent = holder.getAttribute("data-tip"); box.hidden = false;
      const r = holder.getBoundingClientRect(), b = box.getBoundingClientRect();
      box.style.left = `${Math.max(8, Math.min(innerWidth - b.width - 8, r.left + r.width / 2 - b.width / 2))}px`;
      box.style.top = `${r.top - b.height - 6 < 56 ? r.bottom + 6 : r.top - b.height - 6}px`;
    };
    for (const type of ["pointerover", "focusin"]) document.addEventListener(type, e => show(e.target));
    document.addEventListener("pointerout", e => { if (!e.relatedTarget?.closest?.("[data-tip]")) box.hidden = true; });
    document.addEventListener("focusout", () => { box.hidden = true; });
    addEventListener("scroll", () => { box.hidden = true; }, {passive:true, capture:true});
  }

  function bind() {
    $("first").onclick = () => player.go(0);
    $("prev").onclick = () => player.step(-1);
    $("next").onclick = () => player.step(1);
    $("last").onclick = () => player.go(Infinity);
    $("play").onclick = () => player.toggle();
    $("scrub").addEventListener("input", e => player.go(Number(e.target.value)));
    $("mode-single").onclick = () => { comparing = false; render(); };
    $("mode-compare").onclick = () => { if (analysis.pairs.length) { comparing = true; render(); } };
    $("tab-state").onclick = () => setTab("state");
    $("tab-log").onclick = () => setTab("log");
    for (const id of ["tab-state", "tab-log"]) $(id).addEventListener("keydown", e => {
      if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
      e.preventDefault(); e.stopPropagation();
      const next = tab === "state" ? "log" : "state"; setTab(next); $(`tab-${next}`).focus();
    });
    $("filter").addEventListener("input", () => render());
    $("lanes").addEventListener("click", e => {
      const target = e.target.closest("[data-path]");
      if (!target || target.getAttribute("role") !== "button") return;
      const path = JSON.parse(target.getAttribute("data-path"));
      selected = selected && key(selected) === key(path) ? null : path; render();
    });
    $("lanes").addEventListener("keydown", e => {
      if ((e.key === "Enter" || e.key === " ") && e.target.matches("[data-path][role=button]")) { e.preventDefault(); e.stopPropagation(); e.target.dispatchEvent(new MouseEvent("click", {bubbles:true})); }
    });
    $("open").onclick = () => $("file").click();
    $("file").addEventListener("change", e => { openFile(e.target.files[0]); e.target.value = ""; });
    $("save-json").onclick = () => download(`${slugify(trace.title)}.trace.json`, "application/json", JSON.stringify(trace, null, 2));
    $("save-html").onclick = exportHTML;
    $("theme").onclick = () => {
      const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
      document.documentElement.dataset.theme = next;
      try { localStorage.setItem("cp-theme", next); } catch {}
    };
    let depth = 0;
    addEventListener("dragenter", e => { if (e.dataTransfer?.types.includes("Files")) { depth++; $("drop").hidden = false; } });
    addEventListener("dragleave", () => { if (--depth <= 0) { depth = 0; $("drop").hidden = true; } });
    addEventListener("dragover", e => e.preventDefault());
    addEventListener("drop", e => { e.preventDefault(); depth = 0; $("drop").hidden = true; openFile(e.dataTransfer?.files[0]); });
    document.addEventListener("keydown", e => {
      const t = e.target;
      if (t.isContentEditable || ["INPUT", "SELECT", "TEXTAREA"].includes(t.tagName)) return;
      const action = T.keyAction(e);
      if (!action || (action === "toggle" && t.tagName === "BUTTON")) return;
      e.preventDefault();
      ({next:() => player.step(1), previous:() => player.step(-1), first:() => player.go(0), last:() => player.go(Infinity),
        toggle:() => player.toggle(), "next-divergence":() => divergence(1), "previous-divergence":() => divergence(-1)})[action]();
    });
  }

  function boot() {
    bind(); bindTip();
    try { samples = JSON.parse($("trace-samples").textContent || "[]"); } catch { samples = []; }
    let data = null;
    try { data = JSON.parse($("trace-data").textContent || "null"); } catch (error) { toast(`Embedded trace is not valid JSON: ${error.message}`, true); }
    const live = T.liveTrace(window, (() => { try { return sessionStorage; } catch { return null; } })());
    let opened = false;
    if (live) { try { opened = tryLoad(JSON.parse(live)); } catch (error) { toast(`Live trace is not valid JSON: ${error.message}`, true); } }
    const wanted = Math.max(0, samples.findIndex(s => slugify(s.title) === hashParam("sample")));
    if (!opened && !(data && tryLoad(data)) && samples.length) tryLoad(samples[wanted], wanted);
    if (!trace) return void $("status").replaceChildren(el("div", {class:"status-text"}, el("div", {class:"status-title", text:"No trace loaded"}), el("div", {class:"status-detail", text:"Open a JSON or HTML trace, or drop a file anywhere on this page"})));
    const step = Number(hashParam("step"));
    if (Number.isInteger(step) && step > 0) player.go(step - 1);
  }
  boot();
})();

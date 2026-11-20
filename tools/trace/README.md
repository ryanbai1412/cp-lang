# Trace explorer

Open `tools/trace/index.html` directly in a browser. No build, server, fonts,
CDN, telemetry, or network connection is needed. The shadcn/ui zinc-based
visual system matches the documentation: white and neutral surfaces, crisp
borders, system sans/monospace stacks, and blue interaction states.
Light/dark mode follows your system initially and persists locally.

Five deterministic illustrative traces are embedded: a deliberately broken
0/1 knapsack versus a correct spec, Dijkstra, postorder subtree sizes, maze
BFS, and insertion sort. These are generated executions of the example
algorithms, **not captures from the unfinished CP Lang interpreter**.

## Try a capture

```sh
python3 tools/trace/emit.py tools/trace/samples/knapsack.json /tmp/knapsack-trace.html
# Open /tmp/knapsack-trace.html in any modern browser, including offline.
```

Use **Open trace** to load another JSON capture locally. **Save JSON** keeps
the full capture; **Save HTML** embeds the loaded trace in a portable file.
The latter deliberately omits the demo library and docs navigation so the
export remains a standalone capture. No data leaves the browser.

For all documentation, the example playground, and this debugger in one
offline file, run `python3 site/bundle.py` and open `dist/cp-lang.html`.

The timeline supports scrubbing, first/last, previous/next, playback and four
speeds. On the page background, Left/Right step, Home/End jump, and Space
plays or pauses. Native input and button keys keep their normal behavior.
The execution log searches labels, stages, locals and all captured values.
Select an event to move the playhead, then switch back to the inspector.
Cells and array elements can be selected with pointer or keyboard.

## Trace contract

`trace-schema.json` is the versioned JSON Schema. The inline `TraceCore`
also validates semantic constraints on import, before replacing a good
trace: unique event/checkpoint ids, valid source lines, declared variables,
rectangular matrices, graph endpoints, rooted/connected/acyclic trees, and
real focus paths. Invalid files leave the current trace available.

Each event is an immutable **full snapshot**, never a mutation patch:

```json
{
  "schemaVersion": 1,
  "title": "A tiny table",
  "source": {"file": "a.cp", "code": "dp[1][1] = 3"},
  "variables": {
    "dp": {"name": "dp", "type": "[Item][Capacity]Int", "kind": "dp",
           "rowLabels": ["empty", "first"], "colLabels": [0, 1]}
  },
  "runs": {
    "fast": {"label": "solve", "steps": [
      {"id": 0, "line": 1, "stage": "Compute", "label": "Write first item",
       "checkpoint": "item:1:capacity:1", "locals": {"i": 1, "c": 1},
       "values": {"dp": [[0, 0], [0, 3]]},
       "focus": {"variable": "dp", "path": [1, 1]}}
    ]}
  }
}
```

The `kind` is the runtime type's **rendering hint**, not a heuristic based
on its name. `array` uses indexed cells and optional numeric bars; `grid`
and `dp` use a domain-labelled matrix; `graph` uses explicit coordinates
or a circular layout; `tree` uses a rooted level layout. `scalar` and `json`
provide readable fallbacks for any other source type. Graph values contain
`nodes: [{id, label, x?, y?, state?}]` and
`edges: [{id, from, to, weight?}]`. Tree edges point parent to child and the
descriptor's `root` defaults to its first node. Graph ids must be unique
after string conversion. Empty containers are supported.

Paths address zero-based **storage positions**, with domain labels kept
separately for display. Missing variables mean out of scope; JSON `null`
means source `none`, never infinity. Encode non-finite values and integers
outside JavaScript's exact range (±9,007,199,254,740,991) as source-form
strings, e.g. `"inf"` or `"9223372036854775807"`. Objects/records are compared
by contents, independent of key order. Array order matters.

## Fast versus spec

Add `runs.slow` with the same snapshot shape and an optional top-level
`compare: ["dp", "answer"]` list. Without that list, all declared variables
are compared, including out-of-scope differences. Locals are never compared.

Two implementations do not take the same number of machine/interpreter
steps. They align **only at shared `checkpoint` names**, not event offsets.
Use a stable semantic name like `cell:1:4` when both implementations have
finished that cell. Names must be unique within a run and shared names must
occur in the same order. Internal steps need no checkpoint. The viewer
holds the last aligned spec state between checkpoints and explicitly says
there is no comparison there. Zero shared checkpoints is not called success.

The viewer computes the first unequal snapshot itself and opens there. It
highlights every unequal cell, offers exact leaf-path/value inspection and
marks mismatching checkpoints on the timeline/log. The knapsack capture
has 34 fast events and 66 spec events; both first disagree at fast step 5,
spec step 9, `dp[1][4]`: **6 versus 3**. This is the first *captured state*
divergence, not a claim about an unrecorded instruction or causality.

## Emitter integration

The Rust interpreter is still a skeleton; `cpl run --trace` is not wired up
by this standalone viewer change. Its producer can use the same seam as
`emit.py`: replace the contents of `<script id="trace-data"
type="application/json">` in `index.html` with one trace document, replace
`trace-samples` with `[]`, and write the HTML. Both scripts must remain.
Escape **every `<` as `\u003c`** in embedded JSON, even inside source code
and strings; ordinary JSON serialization alone permits `</script>` to end
the element. Also escape U+2028/U+2029. `emit.render_html()` does this with
the standard library. The browser export follows the same rule. Trace data
is rendered as escaped text, never evaluated or used as a resource URL.

Imports are limited to 50 MiB and 100,000 events per run. Matrix rendering
is capped at 100×100, arrays at 500 elements, graphs at 500 vertices and
the execution log at a 500-event window. These limits are explicitly shown;
the complete values still export. This snapshot-first v1 favors inspectable
small counterexamples, not multi-gigabyte execution recording.

## Development

```sh
python3 tools/trace/samples.py        # regenerate JSON files + embedded demos
python3 -m unittest discover -s tools/trace -p 'test_*.py'
node --test tools/trace/trace.test.cjs
cargo test                          # repository-wide required check
```

The Node tests execute the actual inline core extracted from the HTML, not
a second implementation. The Python tests exercise safe embedding and the
sample generator. Browser verification additionally covers all renderers,
time travel, spec alignment, file import/export, keyboard use and narrow
layouts. No npm dependencies are needed for the viewer or these tests.

## Live traces from `cpl`

The compiler records real runs in this schema; nothing here is a fixture.

```sh
cpl run a.cp in.txt --trace                 # writes a.trace.html, this viewer with the run in it
cpl run a.cp in.txt --trace --format json   # the trace document on standard output
cpl run a.cp in.txt --trace --format html   # the same page on standard output (the VS Code webview)
cpl stress a.cp --trace                     # on a failure: a.tests/stress-<seed>.trace.html
```

A step is recorded at every change to a top-level variable, every `print`
and every `debug(...)`. `debug("name", x, y)` is a named checkpoint: the
solution and the file's `@brute(main)` fn are lined up wherever both ran
the same one, and `cpl stress` prints the first that differs. `cpl build`
leaves `debug(...)` out of the C++ entirely.

In the browser, `trace()` of `crates/cpl-wasm` returns the same document,
and the playground's **Trace** button opens it here (`trace.html#live`).

# 4. Tooling and the cp command

Status: decided unless a section says *proposed*. Folded from `tooling.md`
(now in `archive/`); this copy is the source of truth.


Everything around the language: file layout, projects and config, samples,
libraries and bundling, compatibility, the build pipeline and the `cp`
command UX. Replaces `libraries.md`.

## 0. Budgets

- **Save → sample results (debug)** — < 200 ms
- **Save → sample results (compiled)** — < 500 ms
- **Stress: first random case** — < 1 s
- **Submission file ready** — < 1 s

Measured today on a real port (`noi01p5`, 10 KB of C++):

- **Transpile (Python prototype)** — 0.04 s
- **`g++ -O2`** — 1.01 s
- **`g++ -O0`** — 1.00 s
- **`g++ -O2` with a precompiled header** — 0.44 s
- **`g++ -O0` with a precompiled header** — 0.34 s

Almost all of gcc's time is parsing `<bits/stdc++.h>`. A precompiled header
gets compiled builds under 500 ms; skipping gcc entirely for debug runs gets
well under 200 ms.

## 1. File system

```
~/cp/                         <- any folder; nothing machine-global required
  codeforces/
    cp.toml                   <- project: target, libs
    2041/
      d.cp
      d.tests/
        1.in   1.out          <- samples (cp fetch or pasted)
        hand-1.in             <- your own cases, no .out = just run
        stress-48213.in       <- saved automatically by cp stress
        stress-48213.out      <- expected output, from the brute
  ioi/
    cp.toml                   <- different target (graders, interactors)
    2024/nile/nile.cp ...
~/code/cp-lib/                <- your library, used by both projects
  lib.toml
  treap.cp
  graph/flow.cp
```

- **A problem is one `.cp` file plus an optional `<name>.tests/` folder.**
  No per-problem directories or boilerplate required.
- **Test cases are files:** `N.in` / `N.out`. A case with no `.out` is
  run and its output shown. Inline `@sample(in, out)` in the source also
  works, which is handy for a quick paste.
- **Failing stress cases are saved as regression tests** with the brute's
  output, so `cp test` re-checks every bug you've ever hit on that problem.
- **Scratch-input workflow:** keep shared inputs `i1`..`i5` in the folder
  and reuse them across problems. `cp run d i1 i2` (or `cp run d i*`) runs
  each file in order with a header and time per file. If `o1` sits next to
  `i1`, the output is checked against it with a diff; otherwise it's
  printed. With `tests = "scratch"` in `cp.toml`, `cp stress` writes a
  shrunk failing case to the next free `iN` instead of `d.tests/`.
- `cp new d` creates `d.cp` from the project's template and an empty
  `d.tests/`. `cp fetch <url>` downloads samples where the judge allows it.

## 2. Config levels

1. **User** (`~/.config/cp/config.toml`): defaults and named library
   locations.
   ```toml
   [libs]
   mylib = "~/code/cp-lib"
   ```
2. **Project** (`cp.toml`, the nearest one up the tree): the target judge
   and the libraries this project uses. One per judge or contest series if
   you like.
   ```toml
   [target]
   cpp = "c++17"
   size_limit = "64KB"
   judge = "codeforces"            # visible as cfg.judge
   [libs]
   lib  = "mylib"                  # from user config
   geo  = { path = "~/code/geometry" }
   team = { git = "github.com/x/team-lib", rev = "a1b2c3" }
   [new]
   template = "template.cp"
   ```
3. **Solution:** the stamp line at the top records the exact compiler and
   library versions used (§5).

No config at all is fine: `cp run d.cp` works anywhere with the defaults
(stdlib only, C++17, 64 KB).

## 3. Modules and libraries

- **A module is one `.cp` file; a library is a folder with a `lib.toml`**
  (name, plus its own `[libs]` if it depends on another library).
  `~/code/cp-lib/graph/flow.cp` is `lib.graph.flow` in a project that names
  it `lib`.
- **Imports:** `use lib.treap` (qualified), `use lib.treap.{Treap}`, `use
  lib.graph.flow as fl`, `use std.segtree.*` (glob, warns on a clash). Name
  clashes are errors that tell you to qualify. No cyclic imports.
- **Prelude:** `print`, `read`, `min`, `max`, `abs`, `List`, `Map`,
  `Set`, `Int`, `Bool`, `Char`, `Str`, `INF`. Everything else needs `use`.
- **`pub` marks the API;** everything else is private to the file.
  `///` doc comments show in `cp explain` and editor hovers.
- **Library modules hold declarations only** (`fn`, `struct`, `enum`,
  `const`): no top-level statements or `var`s. Importing can't run code or
  add hidden state, so multitest stays safe.
- **Generic library code is specialized at each use site,** as fast as
  pasting it in. Operation values from your solution can be passed to
  library structures.
- **Coherence:** `impl Ord for T` lives only with `T` or the trait. Custom
  orderings are `Order` values (`archive/operations.md`), which are unrestricted.
- **Library tests travel with the library:** `@brute`, `@invariant`, laws.
  `cp fntest ~/code/cp-lib` checks the whole library before a contest.

### Stdlib layering

`std` is a library like yours, layered so each layer depends only on lower
ones:

- **0. Intrinsics** — `Buffer<T>`, primitive ops, bit ops, `I128`, raw I/O, hash seed (the only compiler-provided part)
- **1. Core** — `Opt`, `List`, `Str`, ranges, domain helpers, `Order`, `Eq`, `Hash`
- **2. Algebra** — `Monoid`, `Group`, `Semiring`, `Action`, `ModInt`
- **3. Collections** — `Set`, `Map`, `HashMap`, `Heap`, `Pool`, `Bits`
- **4. Structures** — segtree, Fenwick, sparse table, DSU, treap, Li Chao
- **5. Algorithms** — graphs, trees, strings, math, geometry, DP helpers
- **6. Program** — `@multitest`, `@memo`, `@brute`, `@checker`, `protocol`

## 4. Bundling

`cp build` turns a solution plus its libraries into one C++ file:

1. Resolve `use`s through the config roots into a module DAG.
2. Parse and type-check every module used. Library errors point at the
   library line plus the use site that triggered them.
3. Specialize generics and `const` operation values.
4. Tree-shake to the fns, methods and specializations reachable from the
   solution. Tests, brutes, samples, laws and docs are dropped.
5. Emit one C++17 file with module-prefixed names, standard headers only,
   and the stamp at the top.
6. **Size gate:** an error over the target's limit (64 KB by default) and a
   warning at 75%, with a report:
   ```
   d.cpp  23.4 KB / 64 KB
     solution        4.1 KB
     lib.treap       6.8 KB   Treap<Int,Int>: fold, apply, insert, split
     std.io          3.2 KB
     runtime         8.4 KB
   ```
   `--compact` minifies as a last resort.

## 5. Compatibility: the content-addressed store

Libraries keep changing; old solutions must keep building.

- Every build copies each non-stdlib module it used into
  `~/.cache/cp/store/<hash>.cp` (text, tiny), and the solution's stamp
  records the hashes:
  ```
  // cp 0.4.2  lib.treap@3f9a1c  lib.graph.flow@77e0d2
  ```
- **Default: try the current library, fall back to the stamped one.** On
  each build:
  1. If the current library type-checks with this file, use it and update
     the stamp. If anything the file uses changed, print one line:
     `lib.treap changed since last build: split, fold (cp diff-lib d)`.
  2. If the current library breaks this file, build with the stamped
     version from the store and say so:
     `lib.treap's new version breaks d.cp (split: param removed); built
     with 3f9a1c. cp upgrade d to fix`.
  3. `pin` on the stamp line (`// cp 0.4.2 pin`) or `--frozen` always uses
     the stamped versions. `cp submit` can pin automatically.
  A file you're working on tracks your library; an old file never breaks.
  The stamp line is the one place to see and control this (later: an editor
  action "newer lib.treap available: update / diff").
- **Local path libraries work because of the store:** every build snapshots
  the exact contents by hash, uncommitted edits included, so the built-with
  version always exists without any tagging. Solutions name libraries
  (`use lib.treap`), and config maps names to paths, so moving folders or
  machines changes only config. If the library is a git repo, the stamp also
  shows the commit (`lib@a1b2c3+dirty`) for humans; the hash is the key.
- `std` is pinned by the compiler version, and old compiler revisions are
  kept the same way (`--compiler REV`).
- `cp upgrade d.cp` deliberately moves a solution onto the current
  libraries and compiler, showing what breaks.
- `@deprecated("use split_at")` warns new code without breaking old
  solutions.
- `cp vendor d.cp` copies the used modules into `d.vendor/` for sharing
  with someone who doesn't have your store.
- If the store entry is missing (new machine), the build falls back to the
  current library and warns. `cp store sync <dir>` lets you back the store
  up with your solutions, e.g. in their git repo.

## 6. Build modes

- **Debug** — command: `cp run` (default); how: **interpreter, no gcc**; speed of build / run: ~50 ms / ~30x slower than -O2; checks: everything: bounds, overflow, uninit, `none` unwrap, borrow conflicts, `debug()` output
- **Check** — command: `cp run --check`; how: g++ -O1 + PCH + sanitizers; speed of build / run: ~0.5 s / ~2x; checks: bounds, overflow, UB
- **Release** — command: `cp run --release`; how: g++ -O2 + PCH, same file as submitted; speed of build / run: ~0.45 s / 1x; checks: only `assert`

- **The debug interpreter runs the checked program directly** from the
  compiler's typed IR. No C++ is generated, so a sample run is as fast as
  the type checker plus a few ms. Runtime errors are reported in source
  terms with values:
  ```
  error: index out of range at d.cp:14:18
     dist[v.next]   v = 5, V = 1..=5, v.next = none
  ```
- **The interpreter and the C++ backend must agree.** `cp selftest` runs
  every port and stress case through both and diffs them. Any difference
  is a compiler bug.
- **Compiled binaries are cached** by the hash of the generated C++ and the
  flags, so re-running without changes is instant.
- **`cp watch d`** keeps a daemon running: on each save it type-checks,
  runs the samples in the interpreter, then compiles a release build in the
  background for stress and timing. Libraries stay parsed in memory.
- **Rewriting the compiler in Rust** (planned) moves transpile and check
  from Python speed to about 5–10 ms, which matters most for the
  interpreter path.

## 7. Diagnostics and debugging

The end-to-end experience is in `plan/50-debugging.md`.

Every message is part of the product (goal G5): precise span, the problem's
own terms, and the fix as code.

### Compile errors

```
error: `u` is a V, but `w` is indexed by E
  --> d.cp:21:14
   |
21 |     total += w[u]
   |              ^^^^ expected E, found V
   |
   = note: E = 1..=m (edges), V = 1..=n (vertices); both are 1-based, but
           they don't mix
help: did you mean the edge's weight? `e` is an E in scope:
   |
21 |     total += w[e]
```

```
error: `dist[v]` may be none
  --> d.cp:30:9
30 |     ans = max(ans, dist[v])
   |                    ^^^^^^^ ?Int
help: pick what unreachable means here:
   |     ans = max(ans, dist[v] ?? 0)
   |     if let d = dist[v]: ans = max(ans, d)
```

- One root cause gives one error; follow-on errors are suppressed.
- Every error has a stable code (`E0213`), and `cp explain E0213` shows a
  longer explanation with examples.
- Warnings (complexity over 1e9, unused values, float printing without a
  precision) are separate from errors and never block a build.

### Runtime errors (debug and check builds)

```
error: index out of range
  --> d.cp:14:18
14 |     best = dist[v.next]
   |                 ^^^^^^ v.next is none (v = 5 is V.last)
   = values: v = 5, V = 1..=5
   = call stack: solve (d.cp:40) → relax (d.cp:14)
   = input: test 3 of 7, line 12
```

- Overflow, `none` unwraps, failed `assert`s, borrow conflicts and
  exceeded recursion depth all report like this, with the input position.
- Release builds keep the source map, so a crash in a release run still
  prints a source line (no values).

### Automatic stage tracing

A debug run shows how the answer was built, with no prints to add or
remove. Stages are top-level `let`s and the results of top-level fns:

```
$ cp run d i1 --trace
── i1
  n = 5
  a: [V]  = [3, 1, 4, 1, 5]
  pre: [V] = [3, 4, 8, 9, 14]
  span(V.first) -> (lo: 1, hi: 4)
  best(1, 4) -> 8
output:
8
```

- Values are pretty-printed by type: domain-indexed arrays with their
  domain, grids as grids, optionals as `none`, sets as `{2, 5}`, records
  with field names. Long values are truncated with a count (`… 199,995
  more`).
- `--trace f` limits the trace to one fn (every call, args and result);
  `--trace all` shows nested calls as a tree.
- In multitest, each test's trace is grouped under its test number.
- `debug(x)` prints `d.cp:22  x = …` to stderr in local builds and is
  removed from submissions.
- On a stress failure, `cp stress` prints the trace of the fast solution and
  the brute side by side for the shrunk input, so the first stage that
  differs is highlighted.

### Stage snapshots

A stage's inputs (its arguments, the top-level `let`s it reads, and the
input bytes it consumes, which local builds record) are a complete,
replayable snapshot.

```
$ cp run d i1 --trace
  best(1, 4) -> 8            [s3]
$ cp snap s3
  saved .cp/snaps/best-1.snap   (args l, r; reads n, a)
$ cp run d --stage best-1           # rerun only best on the saved inputs
$ cp run d --stage best-1 --brute   # compare with best_slow
```

- Each traced call has a short id; `cp snap <id>` saves it, `--snap best`
  saves every call to `best`.
- Snapshots are readable text in `print`'s format (domains with sizes,
  arrays, records), so they can be shrunk by hand.
- `--stage` skips input reading and earlier stages; `--trace` works inside.
- `cp locate` and `cp stress` save a snapshot of the part they blame.
- `cp snap keep best-1` turns it into a regression case for `cp fntest`.
- Any fn can be snapshotted. A fn that reads input gets its recorded bytes
  back on replay, and printed output is part of its result.

### Feel

- Colour, aligned output, and one line per success. Failures expand.
- Times shown per run; slow runs flagged against the time limit in
  `cp.toml`.
- Nothing prints a C++ error. If generated C++ fails to compile, that's a
  compiler bug and is reported as one, with the source line.

## 7b. Smart shrinking

- **Structural, not byte-level:** the shrinker knows the `read()` types. It
  removes array elements, lowers `n` (renumbering the domain), deletes
  queries and multitest cases, and lowers values toward 0 or a range's lower
  bound. Constraints (`l <= r`, distinct, tree shape) stay valid at every
  step.
- **Order:** big cuts first (halve sizes), then single deletions, then
  value shrinking; repeat until no smaller case still fails.
- **Live:** a status line shows the current smallest case
  (`smallest: n = 4, q = 1 (from n = 37, q = 12) · 0.3 s`), and the file is
  rewritten on each improvement, so you can stop at any time.
- **Same bug only:** a candidate counts only if it fails the same way
  (same first differing stage, or same crash line).
- **Parallel** on all cores, using a compiled build.
- **Result:** printed, saved to `iN` with the expected output, plus a stage
  snapshot for `--stage` reruns.

## 8. Command UX

```
$ cp test d
  1  ok     3 ms
  2  WA     2 ms
     expected: 7
     got:      6
     first diff: line 1, token 1
  stress-48213  ok
  2/3 passed (debug)

$ cp stress d
  brute: slow (n <= 8)      checker: none
  2,000 cases, 8 threads ... FAIL on case 412 (seed 48213)
  shrunk to n = 3 in 0.2 s:
    3
    1 5 2
  expected 4, got 5
  part that disagrees: best(l: 1, r: 3)   (best_slow says 4)
  saved d.tests/stress-48213.in/.out

$ cp submit d
  d.cpp  12.1 KB / 64 KB  copied to clipboard
```

- **`cp new d`** — create from template, with `d.tests/`
- **`cp fetch <url>`** — download samples
- **`cp run d [files...]`** — run on each file (`i1 i2`, `i*`) or stdin; checks against `oN` if present (debug by default)
- **`cp test d`** — all cases in `d.tests/` + inline samples
- **`cp stress d`** — random inputs vs brutes / subtasks / checker; shrinks; saves
- **`cp fntest d`** — per-fn brutes, invariants, laws
- **`cp locate d`** — minimal failing input + the part to blame
- **`cp snap <id>` / `cp run d --stage <snap>`** — save a stage's inputs / rerun only that stage
- **`cp explain d [name]`** — representations, byte sizes, complexity, docs, name origins
- **`cp watch d`** — rebuild and test on save
- **`cp submit d`** — final C++, size report, clipboard
- **`cp upgrade d` / `cp vendor d`** — move to current libs / bundle libs
- **`cp diff-lib d`** — what changed in the library fns this file uses
- **`cp fmt`** — canonical formatting
- **`cp selftest`** — interpreter vs C++ backend agreement

Every command accepts `d`, `d.cp` or a path, and defaults to the most
recently edited `.cp` file in the folder, so mid-contest you can type just
`cp test`.

## 9. Editor support: LSP and VS Code extension

A top-quality language server plus a VS Code extension, built on the same
compiler daemon as `cp watch`, so the editor and the CLI never disagree.

**Language server (`cp lsp`)**
- Diagnostics as you type, with the same messages as the CLI and every
  suggested fix available as a quick fix.
- Hovers: type, range, domain (with its declaration), byte size and
  representation, and doc comments. On a decorator, what it expands to.
- Inlay hints: inferred types and domains on `let`s and lambda params,
  parameter names at call sites, and `mut` on mutable borrows.
- Go to definition / references / rename across the solution and
  libraries, including into `std`. Rename knows domains, fields and
  methods.
- Completion that ranks by expected type: after `dist[`, values of the
  right domain come first.
- Semantic highlighting that colours each domain consistently, so `V` and
  `E` indices look different on screen.
- Formatting (`cp fmt`) and folding.
- Target latency: diagnostics under 100 ms after a keystroke on a typical
  solution; achievable only with the Rust compiler and incremental
  checking.

**VS Code extension**
- Code lenses on the file and on each `@brute` fn: Run samples, Stress,
  Fntest, Trace.
- A test panel listing `d.tests/` or scratch inputs `i1`..`iN`, with
  pass/fail and diffs.
- The trace walkthrough (`plan/50-debugging.md`) opens in a VS Code webview next to
  the source, synced with the editor cursor: click a trace event to jump to
  its line, or put the cursor on a line to see its last values.
- The stamp line shows "newer lib.treap available: update / diff".
- A status bar item showing the generated size against 64 KB and the last
  run's time.
- Snippets come from your template and library.

Other editors (Neovim, Helix, Zed) get the same LSP; only the trace panel
and test panel are VS Code-specific at first.

## 10. Open questions

- **The binary's name.** `cp` collides with the Unix copy command, so the
  installed binary needs another name (doc 11.1); `cp` stays in the plan
  for readability.

1. Debug interpreter as the default for `cp run`, or compiled with PCH by
   default? (I'd default to the interpreter for samples and use compiled
   builds for stress.)
2. Should `cp submit` also open the judge's submit page or submit directly
   (judge APIs and logins)? I'd leave this out at first.
3. Glob imports, shadowing a `std` module with your own, and a personal
   extended prelude: allow, with warnings and `cp explain` showing origins?

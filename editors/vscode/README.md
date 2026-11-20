# CP Lang for VS Code

VS Code support for **CP Lang v2.1** (`.cp` files): a zero-stale-token TextMate grammar, high-signal contest snippets, `cpl lsp` integration (diagnostics, hover, go-to-definition, document symbols, quick fixes, and inlay hints), CodeLens actions, live inline sample values, counterexample overlays, and `cpl stress` / `cpl trace` commands.

The language surface follows `docs/one-page-cheat-sheet.md` and `research/language-architecture-v2.1.md`; editor UX follows `plan/60-editor.md`.

## Install and develop

Build `cpl` once from the repository root, then install and test the extension:

```bash
cargo build --release -p cpl-cli
cd editors/vscode
npm install
npm test
```

Launch an Extension Development Host directly against the repo:

```bash
code --extensionDevelopmentPath="$PWD" ../../stdlib
```

Package and install a local `.vsix` (build artifacts `*.vsix` stay `.gitignore`d):

```bash
npm run package
code --install-extension cp-lang.vsix
```

To run the full VS Code extension-host integration suite against the real `cpl lsp`:

```bash
npm run test:integration
```

Set `VSCODE_EXECUTABLE` if you want `test:integration` to use an existing VS Code or VSCodium binary instead of downloading one.

## Syntax highlighting (CP Lang v2.1)

`syntaxes/cp.tmLanguage.json` highlights the full CP Lang v2.1 grammar with zero stale v1 tokens (`Float`, `I32`, `U8`, `ModR`, `INF`, `by`, `with`, `static`, `protocol`, `decorator`, `do`, `yields`, `requires`, `ensures`, `pure`, and `<=>` are removed):

```cp
domain V = 1..=n
let xs = compress(raw_vals)                      // Compressed<Int>
var st = LazySegTree<V, Int>(AddSum, a)          // stdlib generic types
var seen = Marks<V>()
type Z = Mod<998_244_353>
var comb = Comb<998_244_353>()

for mut b in blocks:
    defer dsu.undo_to(snap)
    invariant b.l <= b.r
    best min= b.cost ?? 0
    best max= b.gain

fn slice(self, s: Range<V>) lends mut [V]Int:
    law disjoint(s, t) = s.r <= t.l or t.r <= s.l

let x = a.raw[i + 1]                             // escape hatches use unsafe scopes
unchecked:
    total += a.raw[i]

print("ans = {best}, avg = {avg:.6f}")           // interpolation + format specs
```

Highlights worth noting:

- All 40 v2.1 keywords (32 contestant + 8 library-author) and contextual `as` / `from` / `invariant` are kept in lockstep between `src/languageData.ts` and `syntaxes/cp.tmLanguage.json`.
- Builtin types (`Int`, `U64`, `I128`, `F64`, `Bool`, `Char`, `Str`, `Bitset`, `List`, `Deque`, `Map`, `HashMap`, `Set`, `HashSet`, `SortedBag`, `MinHeap`, `MaxHeap`, `Range`, `Bounds`, `Subset`, `Iter`) and standard library contest types (`Mod<P>`, `Comb<P>`, `Marks<D>`, `Compressed`, `MonoDeque`, `SegTree`, `LazySegTree`, `SparseTable`, `Dsu`, `Rooted`, and friends) receive dedicated scopes, including nested generics and turbofish (`f::<Int>()`).
- Boundary properties `.l`, `.r`, `.l(k)`, and `.r(k)` use `variable.other.property.boundary.cp`; mutation (`var`, `mut`, `for mut`, `lends mut`) and escape hatches (`.raw`, `unchecked`) carry dedicated scopes so themes can emphasize them.
- String interpolation `"{expr}"`, format specifiers `"{x:.6f}"`, escaped braces `"{{...}}"`, and raw strings `r"..."` never swallow following lines.

## Contest snippets

`snippets/cp.json` contributes ready-to-type CP Lang v2.1 templates (no stale v1 syntax):

- `memo` / `memohash` — `@memo` and `@memo(hash)` DP templates
- `bsearch_first` / `bsearch_last` — binary search over ranges
- `compress` — coordinate compression with `xs.D`, `xs[r]`, and `xs.at(v)`
- `segtree` / `segtree-monoid` / `lazysegtree` / `sparsetable` — domain-indexed range structures with built-in operator and action choices (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`)
- `rooted` / `hld` — `Rooted<V>` with `LazySegTree<t.Pre, ...>` path and subtree queries
- `dsu` / `marks` / `monodeque` / `mod` — rollback DSU with `defer`, `Marks<V>`, `MonoDeque`, `Mod<P>` + `Comb<P>`
- `multitest` / `grader` / `checker` / `interactor` / `stress` / `atgen` — multi-test skeletons, IOI `@grader` declarations, `@checker`, `@interactor`, and `read(gen: ...)` + `assume` + `@brute(main)` stress setups

## Language server (`cpl lsp`) and inlay hints

The extension launches `cpl lsp` over stdio (`cp.server.path`, default `cpl`, searched on `PATH` and then in workspace `target/release` and `target/debug`).

- **From `cpl lsp`:** live diagnostics with error codes and related spans, hover types, go-to-definition, document outline symbols, and diagnostic quick fixes.
- **Fallback completion and inlay hints:** when `cpl lsp` does not advertise `completionProvider` or `inlayHintProvider` (or before `cpl` is built), the extension supplies v2.1 keyword/type/decorator completions and inferred type inlay hints (`cp.inlayHints.enabled`, default `true`) for unannotated container bindings (`SegTree<V, Int>`, `LazySegTree<V, Int>`, `Rooted<V>`, `Marks<V>`, `compress(...)`) and declared `domain` loops (`for v in V:`). As soon as `cpl lsp` advertises its own provider, the fallback automatically steps aside so hints never duplicate.

## Running, live values, stress testing, and traces

CodeLenses (`cp.codeLens.enabled`, default `true`) appear at the top of `.cp` files, above inline `@sample(...)` declarations, and above `@brute`, `@checker`, and `@interactor` functions:

```cp
▷ Run on i1 | Run on all 3 samples | Stress | Trace
let n: 1..=2e5 = read()                          i1: 3
domain V = 1..=n
let a: [V]Int = read()                           i1: [1, 2, 3]

Stress main against slow | Trace
@brute(main)
fn slow():
    ...
```

Sample discovery checks scratch files `i1`/`o1`, `<stem>.tests/*.in`, `<stem>.samples/*.in`, and inline `@sample("in", "out")`.

- **Run on a Sample / Run on All Samples (`cp.runSample`, `cp.runAllSamples`):** runs `cpl run <file> --format json` (automatically falling back to plain `cpl run <file>` if the installed `cpl` binary does not accept `--format json` yet) and diffs stdout with judge whitespace normalization.
- **Live Sample Values (`cp.toggleLiveValues` / `cp.liveValues.enabled`):** re-runs on save against `cp.liveValues.input` (or the first sample) and renders top-level values as end-of-line virtual decorations with full values on hover.
- **Stress Test (`cp.stress`):** runs `cpl stress <file> --format json` (plus any extra flags in `cp.stress.args`, such as `["--cases", "1000", "--size", "10"]`). Accepts both `"status": "ok"` and `"status": "pass"`, formats counterexamples in the output channel, and paints virtual decorations on line 0 and any differing stages.
- **Trace a Sample (`cp.trace`):** runs `cpl run <file> --trace [part] --format html` and opens self-contained HTML traces in a side-by-side VS Code webview.
- **Clear Overlays (`cp.clearOverlays`), Restart Language Server (`cp.restartServer`), Show Output (`cp.showOutput`):** available from the Command Palette and the language status item.

### CLI JSON contract

When `cpl` emits JSON, the extension consumes:

```text
$ cpl run d.cp --format json < i1
{"values": [{"line": 4, "name": "(n, k)", "value": "(3, 4)"}],
 "stdout": "4\n", "time_ms": 3}

$ cpl stress d.cp --format json
{"status": "ok", "cases": 1000}

$ cpl stress d.cp --format json
{"status": "fail", "cases": 17,
 "counterexample": {"case": 17, "seed": 42, "verdict": "wrong_answer",
   "input": "n = 2  a = [7, 7]", "expected": "1\n", "got": "0\n",
   "stages": [], "saved": "d.tests/stress-17.in"}}
```

Stage-level diffs (`stages: [{"name": "...", "line": 12, "fast": "...", "brute": "..."}]`) highlight the first differing stage across the whole line and mark agreeing stages in green. Plain-text output from `cpl` is printed directly to the `CP Lang` output channel.

## Layout and tests

- `syntaxes/cp.tmLanguage.json`, `language-configuration.json`, `snippets/cp.json` — grammar, indentation/bracket config, and contest snippets.
- `src/languageData.ts`, `src/inlayHintsCore.ts`, `src/samples.ts`, `src/results.ts`, `src/cpl.ts` — pure Node-testable modules (no `vscode` import).
- `src/extension.ts`, `src/client.ts`, `src/completion.ts`, `src/inlayHints.ts`, `src/lenses.ts`, `src/overlays.ts`, `src/runner.ts` — VS Code wiring.
- `test/grammar/*.cp` — `vscode-tmgrammar-test` scope assertions (including `test/grammar/v2_1.cp`).
- `test/unit/*.test.ts` — grammar/v2.1 table parity, corpus tokenization across `proto/`, `stdlib/`, and `examples/local-lib/`, snippet validation, inlay hints, sample/lens planning, stress/run JSON parsing, and real `cpl lsp` stdio protocol checks.
- `test/integration/suite.ts` — end-to-end VS Code extension host suite.

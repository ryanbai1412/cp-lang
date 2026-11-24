# CP Lang for VS Code

VS Code support for **CP Lang 0.2.2** (`.cp` files): an ultra-lean 34-keyword TextMate grammar with zero stale tokens, high-signal contest snippets, `cpl lsp` integration (diagnostics, hover, type-aware completion, signature help, inlay hints, go-to-definition, find-references, rename, document symbols and quick fixes), CodeLens actions, live inline sample values, counterexample overlays, and `cpl stress` / `cpl trace` commands.

The language surface follows `docs/one-page-cheat-sheet.md`, `research/archive/architecture/language-architecture-v2.2.md`, and `decisions.md` §16–§19; editor UX follows `plan/60-editor.md`.

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

## Syntax highlighting (CP Lang 0.2.2)

`syntaxes/cp.tmLanguage.json` highlights the locked **CP Lang 0.2.2** surface with zero stale v1 or removed v2.1 tokens (`defer`, `assume`, `where`, `inline`, `lend`, hard keyword `law`, `for mut`, `Float`, `I32`, `U8`, `ModR`, `INF`, `by`, `with`, `static`, `protocol`, `decorator`, `do`, `yields`, `requires`, `ensures`, and `pure` are removed):

```cp
domain V = 1..=n
let xs = compress(raw_vals)              // Compressed<Int>
var st = LazySegTree<V, Int>(AddSum, a)  // stdlib generic types
type Z = Mod<998_244_353>
let comb = Comb<Z>()  // interior-mutable combinatorics

for d in V:
    invariant d.l <= d.r
    best min= cost[d] ?? break  // diverging RHS on ??
    best max= gain[d]

fn slice(self, s: Range<V>) lends mut [V]Int:
    @law disjoint(s, t) = s.r <= t.l or t.r <= s.l

@interactor
fn judge(sol: Interactor):
    sol.send(n)
    let guess: Int = sol.recv()
    assert guess == secret

let x = a.raw[i + 1]  // escape hatches use unsafe scopes
unchecked:
    total += a.raw[i]

print("ans = {best}, avg = {avg:.6f}")  // interpolation + format specs
```

Highlights worth noting:

- All **34 0.2.2 reserved keywords** (30 contestant + 4 library-author: `trait`, `impl`, `lends`, `yield`), contextual `as` / `from` / `invariant` / `law` / `yields` / `do`, file-level directives (`@!multitest`, `@!...`, scoped `keyword.control.directive.cp`), and item attributes (`@memo`, `@brute`, `@checker`, `@interactor`, `@law`, `@grader`, `@tags`, `@gen`, `@get`, `@test`, scoped `entity.name.function.decorator.cp`) stay in lockstep between `src/languageData.ts` and `syntaxes/cp.tmLanguage.json`.
- Compiler built-in types (`Int`, `U64`, `I128`, `F64`, `Bool`, `Char`, `Str`, `List`, `Map`, `HashMap`, `Set`, `Range`, `Bounds`, `Iter`, `Subset`, `Interactor`), every public `stdlib/` type (`Mod<P>`, `SegTree`, `LazySegTree`, `Deque`, `Bitset`, `Rooted`, ...), the prelude's lazy actions (`AddSum`, `AssignMin`, ...), prelude functions and the `stdlib/` functions that need a `use` each get their own scope, including nested generics and turbofish (`f::<Int>()`). `test/unit/languageData.test.ts` reads `crates/cpl-check/src/prelude.rs`, `stdlib/*.cp` and `stdlib/prelude.cp`, and fails on any phantom or missing name.
- Tropical Optionals (`?T`, `??Int`, `?[V]Int`, `min=`, `max=`, `??=`) receive dedicated `meta.type.optional.cp` / `meta.operator.tropical.cp` scopes distinct from postfix `?` and binary `??`.
- Boundary properties `.l`, `.r`, `.l(k)`, and `.r(k)` use `variable.other.property.boundary.cp` (without misreading open-ended ranges like `..l` / `..r`); mutation (`var`, `mut`, `lends mut`) and escape hatches (`.raw`, `unchecked`) carry dedicated scopes so themes can emphasize them.
- String interpolation `"{expr}"`, format specifiers (`"{x:.5}"`, `"{n:05}"`, `"{n:x}"`, `"{n:b}"`, `"{x:.6f}"`, `"{best ?? 0.0:.3f}"`), escaped braces `"{{...}}"`, and raw strings `r"..."` never swallow following lines.

## Contest snippets

`snippets/cp.json` contributes ready-to-type CP Lang 0.2.2 templates with zero stale v1/v2.1 constructs:

- `memo` / `@memo` — bare `@memo` DP template (auto-dispatches flat vs salted hash table)
- `tropical` / `dist` / `dijkstra` — niche-packed `[V]?Int = none` table with `min=` and `?? -1`
- `bsearch_first` / `bsearch_last` — binary search over ranges
- `compress` — coordinate compression with `xs.D`, `xs[r]`, and `xs.at(v)`
- `segtree` / `segtree-monoid` (`Monoid`) / `lazysegtree` / `lazysegtree-action` (`Action`) / `sparsetable` — domain-indexed range structures with built-in operators/actions and 0.2.2 `Action(id, (new_f, old_f) => ..., (f, val, span) => ..., over: op)`
- `rooted` / `hld` — `Rooted<V>` with `LazySegTree<t.Pre, ...>` path and subtree queries
- `dsu` / `touched` / `small-to-large` / `mod` — explicit `dsu.mark()` / `dsu.undo_to(snap)`, 1-byte `[V]Bool` + touched-list reset, `swap` + `.take()` small-to-large merge, and `Mod<MOD>` + immutable `let comb = Comb<Z>()`
- `multitest` / `@!multitest` / `@brute` / `grader` / `checker` / `interactor` / `stress` / `atgen` — file-level `@!multitest` divider, standalone `@brute(main)` oracle, stateful IOI `@grader`, `@checker`, explicit `sol: Interactor` `@interactor`, and `read(gen: ...)` + `@brute(main)` stress setups

## Language server (`cpl lsp`) and inlay hints

The extension launches `cpl lsp` over stdio (`cp.server.path`, default `cpl`, searched on `PATH` and then in workspace `target/release` and `target/debug`).

- **From `cpl lsp`:** live diagnostics with error codes and related spans, hover types, go-to-definition, find-references (Shift+F12), rename (F2, refused for library names), document outline symbols and diagnostic quick fixes.
- **Completion and signature help from `cpl lsp`:** `xs.` lists the receiver's fields, `@get` properties (shown as properties, so `q.len` never gets parentheses) and methods, even on a half-typed line; bare names list locals, this file's declarations, imports and prelude exports with signatures and `///` docs; `use std.` lists modules. Typing `(` or `,` shows the parameter names, types and the argument being typed.
- **Inlay hints from `cpl lsp`:** the inferred type after each unannotated `let`/`var`/loop variable, and parameter names before literal arguments (`steps: 3`). `cp.inlayHints.enabled` turns them off.
- **Fallback completion and inlay hints:** when `cpl lsp` does not advertise `completionProvider` or `inlayHintProvider` (or before `cpl` is built), the extension supplies 0.2.2 keyword/type/decorator completions and inferred type inlay hints (`cp.inlayHints.enabled`, default `true`) for unannotated container bindings (`SegTree<V, Int>`, `LazySegTree<V, Int>`, `Rooted<V>`, `Rng`, `compress(...)`) and declared `domain` loops (`for v in V:`). As soon as `cpl lsp` advertises its own provider, the fallback automatically steps aside so hints never duplicate.

## Running, live values, stress testing, and traces

CodeLenses (`cp.codeLens.enabled`, default `true`) appear at the top of `.cp` files and above `@brute`, `@checker`, and `@interactor` functions:

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

Sample discovery checks scratch files `i1`/`o1`, `<stem>.tests/*.in` (with `*.out`, the cases `cpl test` runs) and `<stem>.samples/*.in`.

- **Run on a Sample / Run on All Samples (`cp.runSample`, `cp.runAllSamples`):** runs `cpl run <file> --format json` (automatically falling back to plain `cpl run <file>` if the installed `cpl` binary does not accept `--format json` yet), diffs stdout with judge whitespace normalization, shows inline pass/fail verdict badges, and highlights runtime traps inline and in the Problems panel (`cpl (runtime)` diagnostics) with cancellable progress.
- **Testing panel:** every problem with a `<stem>.tests/` folder shows in VS Code's Testing view, one item per case. Running a problem or a case runs `cpl test <file> --format json` (`--case N` for one case); a wrong answer opens as a diff of the expected and the actual output, and a crash, time or memory limit shows `cpl`'s reason.
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

## Settings

Every setting the extension reads; `crates/cpl-cli/tests/vscode_sync.rs` checks this table against `package.json` and the sources.

| Setting | Default | What it does |
|---|---|---|
| `cp.server.path` | `"cpl"` | The `cpl` binary. A bare name is looked up on `PATH`, then in the workspace's `target/release` and `target/debug`. |
| `cp.server.args` | `["lsp"]` | Arguments that start the language server. The server speaks LSP on stdio. |
| `cp.trace.server` | `"off"` | Log the messages between VS Code and the language server. |
| `cp.codeLens.enabled` | `true` | Show Run, Stress and Trace lenses above the file, its samples and its `@brute`/`@check`/`@interactor` functions. |
| `cp.inlayHints.enabled` | `true` | Show inferred type inlay hints for unannotated bindings and domain loops when `cpl lsp` does not provide its own. |
| `cp.liveValues.enabled` | `false` | After each save, run the file on ``cp.liveValues.input`` and show the value of each top-level `let` at the end of its line. |
| `cp.liveValues.input` | `""` | The sample live values are computed on, such as `i1` or `1`. Empty means the file's first sample. |
| `cp.liveValues.maxLength` | `60` | Longest value shown inline before it is cut; the full value is in the hover. |
| `cp.submit.copy` | `true` | Copy the C++ to the clipboard when `CP: Submit` finds it ready to send (`cpl submit --copy`). |
| `cp.stress.args` | `[]` | Extra flags passed to `cpl stress` (for example `["--cases", "1000", "--size", "10"]`). |
| `cp.run.timeoutMs` | `10000` | Kill a sample run or stress run that takes longer than this. |
| `cp.companion.port` | `10043` | The port `CP: Import a Problem` waits on for Competitive Companion (`cpl new --listen --port`); 10043 is one the extension sends to out of the box. |

## Layout and tests

- `syntaxes/cp.tmLanguage.json`, `language-configuration.json`, `snippets/cp.json` — grammar, indentation/bracket config, and contest snippets.
- `src/languageData.ts`, `src/inlayHintsCore.ts`, `src/samples.ts`, `src/results.ts`, `src/cpl.ts` — pure Node-testable modules (no `vscode` import).
- `src/extension.ts`, `src/client.ts`, `src/completion.ts`, `src/inlayHints.ts`, `src/lenses.ts`, `src/overlays.ts`, `src/runner.ts` — VS Code wiring.
- `test/grammar/*.cp` — `vscode-tmgrammar-test` scope assertions (including `test/grammar/v2_2.cp`).
- `test/unit/*.test.ts` — grammar/0.2.2 table parity, corpus tokenization across `proto/`, `stdlib/`, and `examples/local-lib/`, snippet validation, inlay hints, sample/lens planning, stress/run JSON parsing, and real `cpl lsp` stdio protocol checks.
- `test/integration/suite.ts` — end-to-end VS Code extension host suite.

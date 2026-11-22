# 6. Editor support and the language server

Status: proposed. The user asked for "a top notch quality LSP and editor
extensions for vscode at least". This doc says what the editor experience is.

## 6.1 One compiler, two front doors

The language server is the same binary as the compiler (`cp lsp`), kept warm in
memory. There is no second front end,
so the editor and `cp build` can never disagree about an error. When you
type, only the fn you edited is re-checked; everything else is reused.

Targets, measured on corpus files:

- diagnostics for the edited fn in under 100 ms after a keystroke pause;
- hover and completion under 50 ms at p95;
- a cold start on a 500-line file with `std` and `lib` imports under 500 ms;
- no request ever crashes the server: each one is isolated, and a panic
  becomes an internal-error diagnostic with a bug-report link.

## 6.2 What you see while typing

### Diagnostics

Errors appear as you type, cheapest first: syntax, then names, then types,
then domains, borrows and ranges. Broken code is normal, so one missing
`)` gives one error and the rest of the file still gets hover and
completion. Each diagnostic carries its
code, a link to the error index, and its fixes as code actions.

```
let x = deg[e]
            ^ error[E0301]: deg is indexed by V, but e is an E
              help: did you mean deg[edges[e].u] or deg[edges[e].v]?
```

Warnings for cost (likely TLE or MLE against the problem's limits) and
overflow show as soft underlines on the line responsible, and hovering
shows the estimate: "this loop runs about 4e10 times (n * q) with
n, q <= 2e5".

### Hover

Hover answers "what is this and what does it cost?":

```
dist: [V]?Int
  domain V = 1..=n, n <= 2e5
  storage: 8 bytes per element (none uses Int's spare value), 1.6 MB max
```
```
nd: Int
  proven range: 0 ..= 2e14 (d <= 2e14 from dist, w <= 1e9)
  no overflow check needed
```
```
fn dijkstra(src: V) -> [V]?Int
  reads: file-level edges, g
  estimated cost: O((n + m) log m), about 4e6 steps at max input
```

Hover also shows the implicit representation choices the compiler made
(G9): a niche optional, a field width taken from a range, an instantiation
shared with another.

### Inlay hints

- Inferred types on `let` bindings, off by default for obvious literals.
- Parameter names at call sites with more than two positional arguments.
- **Live sample values** (opt-in): each top-level `let` and each part's
  result on the current input `i1`, recomputed by the interpreter in the
  background on save. If a `@brute` exists, the first line whose value
  differs from the brute's is highlighted.

```
let dist = dijkstra(V.first)        // i1: [0, 3, none, 7, 5]
```

### Completion

Completion uses types, so it ranks what fits:

- inside `a[` where `a: [V]T`, values of type `V` come first;
- after `v.`, a `V` offers `next`, `prev`, `before`, `after`, `ord`;
- after `edges[e].`, the record's fields;
- `Sum<`, `Max<` and other operation values when an argument expects a
  `Monoid<T>`;
- stdlib items not yet imported, with an automatic `use std.x` edit.

### Signature help

While typing arguments, the signature appears with the current parameter
highlighted and its bound explained (`m: Monoid<T>`: an identity and an
associative op).

## 6.3 Navigation and refactoring

- Go to definition, including into the stdlib and your `lib/` (the pinned
  snapshot, with a note if your working copy has changed since).
- Find references, document symbols and workspace symbols.
- Rename across files, refusing a rename that would shadow or collide.
- Refactorings (doc 4b): extract fn (computes which file-level `let`s it
  reads and whether it needs `mut`), inline, convert an `Int`-indexed array
  to a domain (the biggest one: it rewrites `a[i + 1]` and asks about
  each ambiguous spot), convert a sentinel to an optional, add a missing
  `mut` at the call site.
- Every diagnostic's fix is a code action; "fix all in file" applies every
  machine-applicable fix.

## 6.4 The VS Code extension

### Highlighting

A TextMate grammar gives instant highlighting before the server starts.
Semantic tokens from the server then refine it: domains, types, operation
values, mutable variables and parameters each get a distinct style, so
`var` bindings and `mut` borrows stand out.

### Running and testing

- A Run lens above the file and above each part: `Run on i1`, `Run on all
  samples`, `Stress`, `Trace`.
- A test explorer listing samples, saved tests, `i1`-`i5`, `@brute` parts
  and stress runs, with pass/fail and time.
- The output panel shows expected vs actual with the first differing line
  highlighted.

### Debugging

The debugger connects through DAP to the interpreter:
breakpoints, stepping, watch expressions, and variables shown with their
domain labels (`v = 3:V`, `s = V[2..=5]`). Because the interpreter records
the run, you can step backwards, and clicking a value in the trace jumps to
the line that wrote it.

### The trace panel

`Trace` opens the HTML walkthrough (doc 5) in a panel beside the source.
Selecting a step in the trace moves the editor cursor to that line, and
selecting a line filters the trace to its steps.

### Status bar

Submission size against 64 KB (green under 48 KB, yellow under 64 KB, red
above), the active judge profile, the compiler version, and whether `lib/`
has changed since this file's pinned snapshot, with a click to upgrade
(doc 4).

## 6.5 Other editors

Any LSP client gets diagnostics, hover, completion, code actions and
formatting. The project ships:

- a tree-sitter grammar, for Neovim, Helix and Zed highlighting;
- small config snippets for Neovim (`nvim-lspconfig`), Helix and Zed;
- a `cp lsp --stdio` mode that needs no configuration.

Running, testing and the trace panel are VS Code features; other editors
use the CLI for them.

## 6.6 Build order

1. TextMate grammar and the `.cp` file association (an hour; useful from
   day one).
2. Diagnostics and formatting through the LSP.
3. Hover with types and ranges, go to definition.
4. Completion and signature help.
5. Run lenses and the test explorer.
6. DAP debugger, trace panel.
7. Refactorings, live sample values.

## 6.7 How it is tested

- LSP requests are tested by fixture files with cursor markers
  (`let x = d$0ist[v]`) and snapshot-checked responses.
- A latency benchmark replays recorded editing sessions on corpus files and
  fails CI if p95 regresses.
- A fuzzer sends random edits and requests and checks the server never
  panics and always answers.

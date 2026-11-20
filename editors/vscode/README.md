# CP Lang for VS Code

Editor support for `.cp` files: highlighting that works before any server
starts, the `cpl lsp` language server for errors, hover, go-to-definition,
the outline and quick fixes, and lenses and inline overlays for running a
file on its samples and reading a failing stress case. The design is
`plan/60-editor.md`.

## Try it

```text
$ cargo build --release -p cpl-cli        # the server the extension starts
$ cd editors/vscode
$ npm install
$ npm test                                # compile, grammar tests, unit tests
$ code --extensionDevelopmentPath="$PWD" ../../proto/ports
```

`npm run test:integration` runs the extension in a real editor against the
real `cpl lsp`. It downloads VS Code; set `VSCODE_EXECUTABLE` to use an
editor you already have (VSCodium works). `npm run package` writes
`cp-lang.vsix`, which `code --install-extension cp-lang.vsix` installs.

## Highlighting

`syntaxes/cp.tmLanguage.json` is a TextMate grammar for the whole surface
syntax of `plan/17-syntax.md`. Reserved words are the lexer's
(`crates/cpl-syntax/src/token.rs`) and the built-in names are the checker's
prelude (`crates/cpl-check/src/prelude.rs`); a unit test reads both Rust
files and fails when the grammar or the completion list falls behind.

A few choices are worth knowing:

```
let x = a.raw[i + 1]            // `.raw` and `unchecked` carry `keyword.other.unsafe`,
unchecked:                      // the scope themes already colour for Rust's `unsafe`
    total += a.raw[i]

fn halves(mut self, k: I) -> lends mut ([I]Int, [I]Int):
    law disjoint(self.l(k), self.r(k))

let from = spec + law           // plain names here: not keywords in this position
```

- `var` and `mut` have their own scopes (`storage.type.var.cp`,
  `storage.modifier.mut.cp`), so a theme can make mutation stand out.
- `.l` and `.r` are `variable.other.property.boundary.cp`.
- Words the lexer does not reserve (`law`, `spec`, `lends`, `requires`,
  `ensures`, `invariant`, `from`, `as`, `do`, `yields`, `static`, `inline`,
  `decorator`, `protocol`) are keywords only where the syntax makes them
  one, and ordinary names everywhere else.
- `Name<` opens type arguments only when `Name` is capitalized and `<`
  touches it, the parser's own rule, so `INF < x` stays a comparison.
- Code inside `"{...}"` is highlighted as code, and `r"..."` has no
  escapes or interpolation.
- `pure fn` is marked deprecated, as the compiler reports it.

## The language server

The extension starts `cpl lsp` and speaks LSP over stdio. `cp.server.path`
names the binary (default `cpl`). A bare name is looked up on `PATH` and
then in the workspace's `target/release` and `target/debug`, so opening
this repository works after `cargo build`. When no binary is found,
highlighting still works and one message says how to fix it.

Everything the server advertises is used as it is: diagnostics with their
code and related locations, hover, go-to-definition, the outline, and each
diagnostic's fix as a quick fix. The server has no completion yet, so
until its capabilities include `completionProvider` the extension offers
reserved words, prelude names and decorators (after `@`) itself; once the
server advertises completion, the fallback is no longer registered.

## Running, live values and counterexamples

Lenses sit above the file, above each inline `@sample`, and above each fn
that has a `@brute`:

```
▷ Run on i1 | Run on all 3 samples | Stress | Trace
@sample("3 4\n1 2 3\n", "4\n")
let (n, k): (1..=2e5, 0..=1e14) = read()        sample 1: (3, 4)
...
Stress against count_brute | Trace
@brute(count_brute)
fn count() -> Int:        fast 1  ≠  brute 0  ← first difference
```

Samples are the ones `plan/40-tooling.md` describes: inline
`@sample("in", "out")`, scratch files `i1`, `i2` (with `o1` as the expected
output) and `<name>.tests/N.in` or `<name>.samples/N.in` with `N.out`.
A run feeds the sample to the program's stdin and compares its output the
way a judge does; the output panel shows the first line that differs.

Live values (`cp.liveValues.enabled`, off by default) rerun the file on one
sample after each save and show each top-level value at the end of its
line. A failing stress run lays the shrunk input over the first line and
marks the part where the fast code first disagrees with the brute. Both
are decorations: they never edit the file, and they go away as soon as the
text changes, because they describe the text that was run.

### What the extension asks of `cpl` (proposed)

`cpl run` and `cpl stress` are not built yet: today `cpl run` checks the
file and exits with status 3, which the extension reports in the output
panel. The overlays are written against this contract, which is the
editor's proposal and not yet reviewed:

```text
$ cpl run d.cp --format json < i1
{"values": [{"line": 4, "name": "(n, k)", "value": "(3, 4)"}],
 "stdout": "4\n", "time_ms": 3}

$ cpl stress d.cp --format json --part count
{"status": "fail", "cases": 313,
 "counterexample": {"case": 313, "input": "n = 2  k = 0  a = [7, 7]",
   "stages": [{"name": "count", "line": 12, "fast": "1", "brute": "0"}],
   "saved": "i3"}}

$ cpl run d.cp --trace count --format html < i1      # the trace page, shown beside the source
```

Lines are 1-based, as in diagnostics. A runtime error is
`"error": {"line": 24, "message": "index out of range"}` in the run report.
Output that is not this JSON is shown as it is, so the plain-text `cpl`
keeps working. `test/fixtures/cpl-stub.js` answers these three commands
for the fixture program and passes everything else to the real `cpl`; the
integration tests use it, and pointing `cp.server.path` at it shows the
overlays before the back end exists.

## Source layout

`src/samples.ts`, `src/results.ts` and `src/cpl.ts` hold the logic and do
not import `vscode`, so `test/unit/` tests them directly. `src/client.ts`
(the server), `src/completion.ts`, `src/lenses.ts`, `src/overlays.ts` and
`src/runner.ts` connect that logic to the editor, and
`test/integration/suite.ts` drives them in an extension host.
`test/grammar/*.cp` are scope assertions, run by `vscode-tmgrammar-test`;
`test/unit/corpus.test.ts` also runs the grammar over every program under
`proto/`.

# The Rust workspace

The front end and both back ends are built: `cpl check`, `cpl run`,
`cpl build` and `cpl lsp` all go through one `cpl_check::Database`, and
`cpl run` and `cpl build` share one typed lowering (`cpl-ir`), so the
interpreter and the C++ cannot disagree about what a program means
without a test saying so. `cpl-verify`, the verification harness, is
built too, and so is the tooling around the compiler: `cpl-fmt`,
`cpl-lint`, `cpl-doc`, `cpl-stress` and `cpl-contest`. The
split is a starting point so that work can go on in parallel without two
people editing the same crate; change it when the code says it is wrong.

## Running and building a program

`cpl run` interprets a program and `cpl build` writes it as one C++17 file:

```text
$ cat sum.cp
let n: 1..=100 = read()
let a: [n]Int = read()
print(sum(a), sum(a) / n, sum(a) % n)
$ printf '3\n-5 1 2\n' | cpl run sum.cp
-2 -1 1
$ cpl build sum.cp                 # writes sum.cpp; `-o -` prints it instead
$ g++ -std=c++17 -O2 sum.cpp -o sum && printf '3\n-5 1 2\n' | ./sum
-2 -1 1
```

`/` and `%` on integers are Euclidean, so a remainder is never negative.
Where the declared ranges prove the dividend is not negative and the
divisor is positive, the C++ is the bare operator (`a % b`); otherwise it
is a branchless helper (`cpl::modp(a, b)` for a positive divisor,
`cpl::emod(a, b)` when nothing is known).

The C++ file opens with the `.cp` source as comments and holds only the
runtime helpers the program calls. If the source would push the file past
the 64 KB a judge accepts, `cpl build` leaves the source out and says so;
if the C++ alone is over, it refuses. Building twice gives the same bytes.

`cpl run` checks what the C++ does not: an index out of range, a division
by zero, an `Int` overflow, `!` on `none`, input outside its declared
range, recursion that never ends. Each stops the program with exit
status 3 and an error that points at the source. A failed `assert` does
the same in both. A `Bool` prints as `YES` or `NO`, as it does in the
prototype. `Str` is bytes and `Char` is one byte everywhere: a
literal must be ASCII (`E0502`), and other bytes pass from input to
output untouched.

## Testing the back ends

`crates/cpl-cli/tests/programs` holds one `.cp` per area of the language,
each with the `.in` it reads and the `.out` it must print.
`tests/differential.rs` runs every one through the interpreter and
through `g++ -std=c++17 -fsanitize=address,undefined`, and fails if the
two differ in output or exit status, if a sanitizer speaks, or if g++
warns. `tests/budget.rs` holds the binary to 200 ms for `cpl run`, one
second for `cpl build` and 64 KB of C++. To add a program, write the
`.cp` (and `.in`), then record its `.out` from the interpreter:

```text
$ CPL_BLESS=1 cargo test -p cpl-cli --test differential
```

Read the recorded `.out` before committing it: from then on it is what
both back ends are held to.

`cpl-verify fuzz` adds random programs to the fixed ones: it checks
`cpl run` and the sanitized C++ against its own reference evaluator
(`crates/cpl-verify/README.md` has the command).

## One front end, reused per item

A file is lexed whole and cut into top-level items. Everything after that
is cached per item. Syntax is keyed by the item's text, with spans relative
to the item, so an item that only moved is not parsed again. A check records
each top-level name it looked up and a fingerprint of what it saw, and is
reused while those fingerprints match. Take this file:

```
fn link(mut g: [V]List<(V, Int)>, r: Road):
    g[r.u].push((r.v, r.len))

for e in E:
    link(mut g, roads[e])
```

Editing the body of `link` re-parses and re-checks `link` alone: its
signature has the same fingerprint, so the `for` loop is not touched.
Adding a parameter to `link` re-checks `link` and the loop, and nothing
else in the file. `crates/cpl-check/src/db.rs` has the details, and
`cpl-check`'s tests count the work each kind of edit causes.

Broken code is the normal case. The lexer, parser and checker never fail,
and one root cause gives one diagnostic: after its first error in a
statement the parser stays quiet until the line ends, and the checker says
nothing about code that already has an error.

To try it:

```
$ cargo run -q -p cpl-cli -- check proto/tests/bad/wrong_domain.cp
$ cargo test --release -p cpl-lsp --test latency -- --nocapture
```

The second command prints the latency of a cold check, a check after one
keystroke, and the language server's diagnostics and hover, on 500-line and
5,000-line files, and fails if any is over budget (50 ms for a check,
100 ms for the server).

The checker covers names, the types of bindings, fns, structs, enums and
domains, strict domain indexing, call arguments, and `mut`. It also covers
what the libraries are written in:

- **Generics.** A type parameter is filled in where it is used. With
  `struct Pair<A, B>(first: A, second: B)`, `Pair(City.first, 'x').first`
  is a `City`, and `pick::<Int>(3, 4)` names the parameter outright.
- **Traits and impls.** An impl must write every fn its trait leaves
  without a body and no fn the trait lacks, a bound must name a trait, and
  `a + b` on a struct needs `impl Add` for it.
- **Modules.** `use std.dsu.{Dsu}` loads `stdlib/dsu.cp`, and a local
  library is found through its `lib.toml`. Only `pub` names can be
  imported, and a missing module or name is one error on the `use`.
  `cpl-check/src/modules.rs` says where modules are looked for.
- **Lends.** Two live lends of one value must be provably apart:
  `swap(mut a[i], mut a[j])` needs `i != j` from a guard, an `assert` or
  an early `return`. A slice bound by `let` keeps its array lent until its
  last use, a lent slice can't be stored or returned, and a projection
  lends by its `law disjoint` (`plan/19-index-and-lends.md`).
- **Euclidean `/` and `%`.** `x % 7` is known to be in `0..=6` whatever
  the sign of `x`, so it indexes a `0..7` domain.

`crates/cpl-cli/tests/corpus.rs` holds this to the source in the
repository: `stdlib/`, `examples/local-lib/`, `proto/ports/` and the
prototype's passing tests check with no diagnostics, and each file in
`proto/tests/bad/` is rejected for the reason its first line states.

Not checked yet: value ranges beyond literals and domains, whether a type
argument satisfies the bounds of the parameter it fills, and laws beyond
`law disjoint`. The prelude is a table in `cpl-check/src/prelude.rs`. It
also lists the names the prototype compiler built in (`SegTree`, `Mod`,
`dijkstra` and others) and accepts them without a type, because the ports
in `proto/ports/` use them without a `use`; a port that imports the `std`
module instead gets the real types.

The pipeline, in the order the crates will depend on each other (add the
dependency when the code first needs it):

- **`cpl-diag`** is diagnostics: spans, stable error codes, rendering.
  Every other crate reports errors through it (`plan/30-compiler.md` 3.7).
- **`cpl-syntax`** is the lexer, the parser and the syntax tree. The
  parser recovers from errors so the editor works on half-written code
  (`plan/00-vision.md` G13, `plan/17-syntax.md`).
- **`cpl-check`** is name resolution, types, domains, borrows and value
  ranges (`plan/11` to `plan/16`, `plan/30-compiler.md` 3.4 and 3.5).
- **`cpl-ir`** lowers a checked program to the typed tree both back ends
  read: names resolved to slots, patterns and comprehensions turned into
  loops, every `read()` given the type it reads. What neither back end
  can run yet is reported here, once, as `E0501`.
- **`cpl-codegen`** turns that tree into one C++17 file (`plan/30-compiler.md` 3.3, `plan/70-judges.md`).
- **`cpl-interp`** runs that tree directly for debug runs, checking
  everything the C++ leaves unchecked (`plan/30-compiler.md` 3.1,
  `plan/50-debugging.md`).
- **`cpl-fmt`** is the formatter behind `cpl fmt`: one canonical style,
  no options (`plan/45-dev-tools.md`).
- **`cpl-lint`** is `cpl lint`: code that is legal but probably wrong,
  slow or unidiomatic, with levels in `cp.toml` and safe fixes.
- **`cpl-doc`** is `cpl doc`: HTML and JSON documentation from `///`
  comments, with checked examples.
- **`cpl-stress`** is `cpl stress`: inputs generated from the `read()`
  types, a brute to compare with, structural shrinking, and checker and
  interactor judging (`plan/40-tooling.md` 7b).
- **`cpl-contest`** is the workflow around one problem: `cpl new`,
  `cpl test`, `cpl bench` and `cpl watch` (`plan/40-tooling.md`).
- **`cpl-lsp`** is the language server (`plan/60-editor.md`).
- **`cpl-cli`** builds the `cpl` binary and owns the commands (`plan/40-tooling.md`,
  `plan/45-dev-tools.md`).
- **`cpl-wasm`** is the same pipeline as a WebAssembly module, for the docs
  site and the playground: check, run, build and hover in the browser with
  no server (`plan/90-docs-and-playground.md`, `crates/cpl-wasm/README.md`).
  `scripts/build-wasm.sh` builds it into `dist/wasm/`.

Beside the pipeline, **`cpl-verify`** is the verification harness: the
differential fuzzer under ASan and UBSan, the `sizeof` audit, the raw and
library-stripped line counts and the latency benchmark
(`plan/80-validation.md`, `crates/cpl-verify/README.md`). It tests
`proto/cpc/` today and the crates above as they land.

**`cpl-prover`** (`crates/prover/`) is the prover for `invariant`, `law`
and the range checker: index bounds, overflow freedom, `law disjoint` and
the algebraic laws of the standard library, each answered with a proof or
a concrete counterexample (`crates/prover/README.md`). It stands alone
until the checker calls it.

```text
$ cargo test            # must pass before every push to main
$ cargo run -p cpl-cli -- --version
cpl 0.0.0
```

Not Rust: the VS Code extension is in `editors/vscode/`. Not here yet: the
standard library and the sample local library (`.cp` source), the trace
viewer and the docs site. Each gets its own top-level directory when it
starts.

# The Rust workspace

The front end and both back ends are built: `cpl check`, `cpl run`,
`cpl build` and `cpl lsp` all go through one `cpl_check::Database`, and
`cpl run` and `cpl build` share one typed lowering (`cpl-ir`), so the
interpreter and the C++ cannot disagree about what a program means
without a test saying so. Around the compiler sit the tools: the formatter, the
linter, docs, stress testing, the contest workflow and the judge profiles. The
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
by zero, an `Int` overflow, `!` on `None`, input outside its declared
range, recursion that never ends. Each stops the program with exit
status 3 and an error that points at the source. A failed `assert` does
the same in both. A `Bool` prints as `YES` or `NO`, as it does in the
prototype. `Str` is bytes and `Char` is one byte everywhere: a
literal must be ASCII (`E0502`), and other bytes pass from input to
output untouched.

An optional is one scalar in the C++ wherever a value is free to say
`none`. A position is an ordinal from 0, so `?V` is an `int` with -1 for
`none`; a `?Bool` is one byte; a `?Char` is sixteen bits, since every byte
is a `Char`. An `Int` has no value to spare, so `?Int` is a
`std::optional<ll>`, 16 bytes, unless the program proves one: a table of
`?Int` is held as 8-byte cells with `Int.MIN` for `none` when nothing
stored in it can be `Int.MIN`.

```text
let w: [Item]1..=1e9 = read()
var best: [Cap]?Int = none            // std::vector<cpl::OptInt>: 8 bytes a cell
best[Cap.first] = 0
for i in Item:
    for c in Cap.rev:
        if let d = c - w[i]:
            if let b = best[d]: best[c] max= b + w[i]
```

Here every value stored is 0 or a stored value plus input declared at
least 1, so none is `Int.MIN`. Store an `Int` read with no range, a
difference, or what a fn returned, and the table stays `std::optional`;
so does one that is passed, returned, compared or looped over as a
whole. The proof trusts what `cpl run` checks (declared input ranges,
no overflow), and `crates/cpl-codegen/src/niche.rs` is all of it.

`@memo` on a fn that can't be remembered is an error (`E0311`), not a fn
that quietly runs every time: one with a `mut` parameter, a method, a
generator, a generic fn, or one that gives no value.

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
$ cargo run -q -p cpl-cli -- check crates/cpl-cli/tests/cases/bad/wrong_domain.cp
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
- **Modules.** `use std.dsu.{Dsu}` loads `std/dsu.cp`, and a local
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
repository: `std/`, the ports in `proto/ports/`
and `proto/ports/hillclimb_dp_graph/`, and the prototype's passing tests
check with no diagnostics, and each file in `crates/cpl-cli/tests/cases/bad/` is
rejected for the reason its first line states. The sketches in
`proto/ports/scout_batch_*/` are left out: they are written in syntax the
language does not have yet. `main` moves quickly, so that file also names
what the checker is behind on: `PENDING` is the passing files it does not
read yet (none now), and `NOT_YET` the failing tests it still accepts.

Not checked yet: value ranges beyond literals and domains, whether a type
argument satisfies the bounds of the parameter it fills, and laws beyond
`law disjoint`.

The prelude is [`std/prelude.cp`](../stdlib/prelude.cp): the one list
of standard library names a program has without a `use`, as `use` lines.

```cp
// no `use std.mod.{Mod}` needed: the prelude lists it
let x = Mod<1_000_000_007>(5)
```

`cpl check`, `cpl run`, `cpl build`, `cpl lsp` and the playground all read
that file, and each resolves a name to the cp source of its module, so
none of them takes a name on trust. `cpl-check/src/prelude.rs` keeps only
what is not cp source: the core types (`Int`, `List`, ...) and the
intrinsics the libraries are written on (`print`, `read`, `sort`, ...).
`Subset<D>` is still lowered by `cpl-ir` rather than declared in `std/`.

Each crate's job, and the stage it belongs to, is in
`handbook/design/architecture.md` (Crates); each crate's `README.md` has its
invariants and where its tests are. `scripts/build-wasm.sh` builds `cpl-wasm`
into `dist/wasm/`.

**`cpl-prover`** (`crates/cpl-prover/`) is the prover for `invariant`, `law`
and the range checker: index bounds, overflow freedom, `law disjoint` and
the algebraic laws of the standard library, each answered with a proof or
a concrete counterexample (`crates/cpl-prover/README.md`). It stands alone
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

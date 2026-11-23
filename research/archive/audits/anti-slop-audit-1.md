# Anti-slop audit 1: compiler, stdlib, tests, tooling

Audited at `5879196` (CP Lang 0.2.2), by reading the code and by running the
built `cpl` on small probe programs. Every claim below was either read at the
cited line or reproduced with the probe shown next to it. Nothing outside this
file was changed.

Ranking: **P0** means the toolchain says something false to the person using
it (a clean check on a program that cannot run, a green number that was not
measured, a judge binary that silently computes garbage). **P1** means a rule
of the design is broken in a way that will produce P0s as the code grows.
**P2** is duplication, stale text and dead weight.

## The short version

The one finding that explains most of the others: **there are two type
checkers, and they do not share a result.** `cpl-check` (10,746 lines) is what
`cpl check` and the editor run. `cpl-ir` (14,413 lines under
`crates/cpl-ir/src/lower/`) is what `cpl run` and `cpl build` run, and it
resolves every name and types every expression again from the syntax tree.
The first is permissive where it has no answer; the second is the only one
that is right. So:

- `cpl check` and the language server accept programs that can never run
  (sections 1 and 2).
- Of the 1,001 `.cp` files under `proto/ports/`, **458 build**, **233 pass
  `cpl check` and then fail `cpl build`**, and **310 fail `cpl check`**. The
  corpus test only runs `check`, and leaves 828 of those files out (section 4).
- The published scoreboard ("566 of 579 problems differentially checked") was
  measured on the Python prototype, not on the Rust compiler (section 4).
- The judge build drops checks that the optimizer's proofs depend on
  (section 5).

How the sweep was run, so it can be repeated:

```sh
cargo build -p cpl-cli
for f in $(git ls-files proto/ports | grep '\.cp$'); do
  target/debug/cpl check "$f" >/dev/null 2>&1 || { echo "checkfail $f"; continue; }
  target/debug/cpl build "$f" -o - >/dev/null 2>&1 && echo "builds $f" || echo "lowerfail $f"
done | cut -d' ' -f1 | sort | uniq -c
```

---

## 1. Hardcoded userland builtins and prelude shortcuts

### 1.1 P0: `cpl-check` types 33 prelude functions as "anything goes"

`crates/cpl-check/src/prelude.rs:32-56`. The doc comment says it plainly:
"Fns that are generic over what they are given are `Unknown`, which accepts
any call." That covers `print read min max abs sum count all any find first
last argmax argmin sort sorted shuffle swap copy rand some laws_hold sqrt
atan2 floor ceil round len counts prefix span take columns`.

```
let a: List<Int> = [1, 2, 3]
let s: Str = sum(a)          # cpl check: no errors
let x: Int = abs("hello")    # cpl check: no errors
print(1, sepp: "x")          # cpl check: no errors
```

`cpl run` then reports `E0302 expected Str, found Int`, `E0302 expected Int,
found Str` and `E0310 print has no argument named sepp`, all from
`cpl-ir`. The editor shows none of them, because `cpl-lsp` depends only on
`cpl-check` and `cpl-diag` (`crates/cpl-lsp/Cargo.toml`).

Fix: these functions get real signatures. Most are expressible as generic fns
over a trait (`sum<T: Add>(xs: Iter<T>) -> T`) and belong in
`stdlib/prelude.cp`; the handful that truly need the compiler (`read`,
`print`) get a typed intrinsic signature in one table that both the checker
and the lowering read.

### 1.2 P0: `PROTOTYPE_NAMES` makes 24 names untyped values *and* untyped types

`crates/cpl-check/src/prelude.rs:87-112`, used at lines 53 (as a value) and 78
(as a type): `gcd lcm runs distinct product compress inf ask dijkstra Rooted
SegTree StrHash MinHeap HashMap AddMax WithMin Perm Subset Mod Graph Dsu
AssignSum condense scc`. The comment says they exist "so those ports still
check".

```
let x = dijkstra(1, 2, 3, "a")
let y = Rooted.whatever
print(inf + compress)        # cpl check: no errors
let s = SegTree(5)
s.nonsense(1, 2, 3)          # cpl check: no errors
```

`cpl run`: `E0203 Rooted has no field or method whatever`, `E0203 SegTree has
no field or method nonsense`. This list is the mechanism by which 233 ports
"pass" the corpus test without being able to run.

Fix: delete the list. A port that needs `Dsu` writes `use std.dsu.{Dsu}` (or
gets it from the single prelude, 1.5). Ports that then fail are failures, and
are counted as such.

### 1.3 P0: `Subset<D>` is a compiler feature spread over five crates, and is not in `stdlib/` at all

- parser special case: `crates/cpl-syntax/src/parser/expr.rs:613`
  (`for mask in Subset<Row>:`)
- checker: untyped, through `PROTOTYPE_NAMES` (`prelude.rs:105`)
- lowering: `crates/cpl-ir/src/lower/mod.rs:1279`, `1358`, `1512`;
  `lower/expr.rs:1416`, `1979-2000`, `2296`; `crates/cpl-ir/src/lib.rs:35`,
  `614`
- interpreter: `crates/cpl-interp/src/lib.rs:509-518`
- C++: `crates/cpl-codegen/src/stmt.rs:24-37`

`grep -l Subset stdlib/*.cp` finds nothing. This is the opposite of the
approved design (one bounded-index mechanism, with `Subset`, `Str`, `Grid`
and `Perm` as userland on top of it), and it is why "iterating a `Subset<D>`
type alias" needed bespoke lowering work.

Fix: write `Subset<D>` in `stdlib/` as a sealed index over `0..2^K` with
`contains`, iteration and `[Subset<D>]T` tables coming from the general
domain machinery; then delete the seven lowering sites, the parser case and
the two backend sizers. What the compiler must still supply (if anything) is
a general "domain derived from a domain" hook, justified in `decisions.md`.

### 1.4 P1: `runs` and friends are compiled by Rust even though `stdlib/seq.cp` defines them

`crates/cpl-ir/src/lower/call.rs:988` dispatches `"runs"` to `lower_runs`
(`call.rs:1274-1385`, 110 lines of hand-built IR), while `STD_NAMES` maps
`("runs", "seq")` (`prelude.rs:319`). Both of these print the same thing, and
nothing says which implementation ran:

```
print(runs([1, 1, 2]))                 # compiler builtin
use std.seq.{runs}
print(runs([1, 1, 2]))                 # stdlib, or still the builtin?
```

The same `prelude()` match (`call.rs:679-1077`) hand-lowers `min`/`max`
(786), `sum`/`count`/`all`/`any` (793), `shuffle` (912), `sort` (945),
`sorted` (976), `first`, `abs` and more, and `method()`
(`call.rs:2792-3590`) is an 800-line table of methods per built-in type.

Fix: move each to `stdlib/prelude.cp` unless a measured reason keeps it an
intrinsic. Keep a short, listed intrinsics layer (`decisions.md` already asks
for this) and make `prelude()` exactly that list.

### 1.5 P1: there is no prelude, there are five tables

The set of "names you get without a `use`" is written down separately in:

1. `prelude.rs:32-56` (`value()`), 2. `prelude.rs:119` (`TYPE_NAMES`),
3. `prelude.rs:151` (`VALUE_NAMES`), 4. `prelude.rs:217` (`STD_NAMES`, name
to module), 5. `crates/cpl-ir/src/lower/call.rs:679-1077` (what actually
runs), plus 6. `editors/vscode/src/languageData.ts:28-64` (what the editor
completes and highlights).

They disagree in both directions:

- Accepted by the checker, unknown to the lowering (`E0201 cpl run doesn't
  know a fn named ...`, `call.rs:1073`): `find last argmax argmin counts
  prefix span take columns distinct product compress`.
- Lowered but rejected by the checker, so unreachable: `clz`
  (`call.rs:842-846`), `bit` (`call.rs:855`).
- Offered by the editor as "prelude fn"/"prelude type" and defined nowhere
  (not in `stdlib/*.cp`, `cpl-check` or `cpl-ir`): `Rng SuffixArray read_line
  read_until read_output debug pow_mod sin cos tan exp ln hypot trunc close
  sign turn_eps unit PI`.

Fix (this is the "proper prelude" item): one file, `stdlib/prelude.cp`,
re-exports what is auto-imported. The checker loads it like any module. The
tables 1-4 are deleted; the editor's list is generated from `cpl doc --json`
(or served by the language server as completion), never typed by hand.

### 1.6 P1: implicit std imports are a text search, and only `run`/`build` do it

`crates/cpl-check/src/db.rs:455-484`: when `implicit_std` is on, each
upper-case name in `STD_NAMES` is looked for as a substring of the file's
text (`item.text.match_indices(name)`, so comments and string literals
count) and imported if found. It is switched on by `cpl run`/`build`
(`crates/cpl-cli/src/lib.rs:201`), the playground and `cpl stress`, and off
for `cpl check` and the language server (`db.rs` doc comment on
`use_std_implicitly`). Result:

```
var d = Dsu(5)      # cpl check: no errors (PROTOTYPE_NAMES, untyped)
                    # cpl run:   E0303 `Dsu` takes 0 arguments, but 1 was given
```

Two tools, two different meanings of the same file. Fix: the prelude decides
(1.5). If `Dsu` should be in scope without a `use`, it is in the prelude for
every tool; otherwise every tool reports the missing `use`.

---

## 2. Check-on-trust, silent drops, stubbed semantics

### 2.1 P0: `@law` is never checked, and a 7,300-line prover is wired to nothing

```
@law comm(a: Int, b: Int) = a - b == b - a
@law nonsense(x: Int) = false
print(1)            # cpl check: no errors;  cpl run: prints 1
```

`crates/prover/` (package `cpl-prover`: `omega.rs`, `smtlib.rs`, `laws.rs`,
`lends.rs`, 7,307 lines) is not a dependency of any other crate. It has its
own parser for "the expression subset of `.cp`" (`crates/prover/src/parse.rs`
header), i.e. a second parser that will drift, and its tests
(`crates/prover/tests/laws.rs:15-30`) retype the structures of
`stdlib/algebra.cp` as string literals instead of reading the file.

Fix: either wire it in (`cpl check` hands each `@law` and `law disjoint` to
the prover using the real AST, and falls back to randomized testing with a
visible "tested, not proved" note), or state in `decisions.md` that laws are
documentation and make `cpl test` at least run them on random values. A law
that is silently ignored is worse than no law.

### 2.2 P0: decorators are accepted by name, arguments unread

`crates/cpl-ir/src/lower/mod.rs:566-568` lets `get brute test solution tags
memo law gen checker interactor grader` through with `=> {}`; `cpl-check`
does not validate decorators at all:

```
@tags("dp", 3, whatever)      # `whatever` is undefined: no error anywhere
@solution
@get(42)
fn f(x: Int) -> Int: x + 1
@memo(size: 3, bogus: 7) fn g(x: Int) -> Int: ...   # arguments ignored
@zzz fn h(x: Int) -> Int: x   # cpl check: no errors; cpl run: E0501
```

Fix: one decorator table (name, where it may appear, argument schema, which
command consumes it) read by the checker. Unknown decorator or bad argument
is a `cpl check` error. Decorators nothing consumes (`@solution`, `@get`,
`@tags` if that is the case) are removed from the language.

### 2.3 P0: 29 language features pass `cpl check` and die in lowering with "can't handle ... yet"

`Lower::unsupported` (`crates/cpl-ir/src/lower/mod.rs:523-529`, code `E0501`)
is called from 29 places. The list, from the message strings: `with`, `<=>`,
`if const`, `while const`, variadic parameters, stepped slices, a stepped
range kept as a value, ranges without a start/end in a `for`, array
dimensions that don't start at 0, names bound inside a `|` pattern, more than
one `if` (or an `if let`) in a comprehension, `in` inside a comparison chain,
explicit type arguments in some positions, some generic fns and types
(`call.rs:334`), `return`/`break` out of a block used as a value, a `Bool`
element passed as `mut`, a pattern as a sort-key parameter, a domain that is
not a range, `@memo` on some fn shapes. In the port sweep, "array dimensions
that don't start at 0" alone stops 12 files.

Fix: for each, implement it or remove it from the parser and `decisions.md`.
Until then the *checker* must report `E0501`, so the editor shows it.

### 2.4 P1: approved escape hatch `a.raw[i]` does not run

```
let a: List<Int> = [1, 2, 3]
unchecked:
    print(a.raw[7])     # cpl check: no errors
                        # cpl run: E0203 a `List<Int>` has no member `raw` that `cpl run` knows
```

### 2.5 P1: fixed-width integer types are aliases of `Int`

`prelude.rs:68`: `"I32" | "U8" | "U64" | "I128" => Ty::Int`.

```
let a: U8 = 300
let b: I32 = 5000000000
let c: U64 = -1
print(a, b, c, a + 1)    # prints: 300 5000000000 -1 301
```

A `U64` hash that is really a signed 64-bit `Int` traps on overflow in
`cpl run` and wraps on the judge. Fix: real types with defined wrapping, or
remove the names until they exist.

### 2.6 P1: the borrow checker decides "does this call mutate" from the method's *name*

`crates/cpl-check/src/check/lends.rs:26-30`:

```rust
const MUTATORS: &[&str] = &["push", "pop", "sort", "rev", "clear", "insert", "remove", "swap", "set", "add"];
const STORES: &[&str] = &["push", "insert", "set", "add"];
```

used at lines 415, 500, 1441, 1601 and 1657. A `mut self` method with any
other name is invisible to it:

```
struct Bag:
    xs: List<Int>
    fn grow(mut self, v: Int):
        self.xs.push(v)
var b = Bag(xs: [1, 2, 3])
for x in b.xs:
    b.grow(x)            # cpl check: no errors
```

whereas `a.push(x)` inside `for x in a` is `E0602`. The program survives
only because the C++ backend runs its own (correct) mutation analysis and
iterates a copy (`std::vector<ll> t0 = b.xs; for (ll x : t0)`): a hole in the
checker hidden by a silent O(n) copy in the backend. The module's own header
says disjointness is decided "never by a guess".

Fix: use the callee's signature (`mut self`, `mut` parameters) through the
resolved method, which requires 3.1. Then the defensive copy becomes an
internal assertion, not a behavior.

---

## 3. Backend and frontend drift

### 3.1 P0 (root cause): `cpl-ir` ignores the checker's result and checks again

`cpl_check::Analysis` (`crates/cpl-check/src/db.rs:128-135`) exposes text,
diagnostics, items and a `pub(crate)` scope: no typed tree, no resolved
names. `cpl-ir` reads only `analysis.items`, `.modules`, `.imports` and
`.aliases`, then emits its own diagnostics from 138 `codes::` sites
(`call.rs` 59, `expr.rs` 38, `stmt.rs` 23, `mod.rs` 12, `mono.rs` 6),
including the user-facing `E0201`, `E0203`, `E0302`, `E0303`, `E0310`. Every
item in sections 1 and 2 is a place where the two disagree.

Fix, and the first cleanup wave: one front end. Either `cpl-check` produces
the typed, name-resolved tree and `cpl-ir` becomes a pure translation that
cannot report a type error (any diagnostic it emits is an internal compiler
error), or `cpl-ir`'s checker becomes the checker and `cpl-check` shrinks to
the incremental database around it. The approved architecture already says
"one incremental Rust frontend" for `run`, `build` and `lsp`.

### 3.2 P0: three implementations of the language, and the tests are split between them

- Rust: `crates/` (what ships).
- `proto/cpc/compiler.py` (7,054 lines, Python to C++) with `runtime.hpp`.
- `proto/cpi/` (a Python interpreter with its own `parser.py` and
  `builtins.py`).

`proto/run_all.sh` (the port regression) runs `python3 cp.py`. The corpus
registry counts the prototype's passes (4.2). `crates/cpl-verify` fuzzes the
prototype (4.3). `stdlib/README.md:30-32` still says "The Rust interpreter
and code generator are not built yet, so the library's tests run on a
reference interpreter, `proto/cpi/`", which is false: `cpl test stdlib/*.cp`
runs all 119 `@test` fns on the Rust interpreter and they pass.

Fix: the prototype is for design experiments only (as memory says). No
number that is published, and no test that gates, may come from it. Move
`run_all.sh`, the registry and `cpl-verify` to the Rust `cpl`, then fix the
README.

### 3.3 P1: the language server sees a fraction of the language

`crates/cpl-lsp/src/server.rs:342-357` advertises hover, definition, document
symbols and code actions. No completion (the VS Code extension fakes it from
a hand-typed list, 1.5), no lowering diagnostics (3.1), no inline execution
or counterexamples (approved as guardrail 5). It also carries its own JSON
implementation (`crates/cpl-lsp/src/json.rs`, 387 lines; a second one lives
in `crates/cpl-verify/src/json.rs`).

### 3.4 P1: the playground skips interactive programs

`crates/cpl-wasm/tests/parity.rs:62-67` and `tests/smoke.mjs:82-85` skip
every program whose text contains `"\n@interactor"`. The parity suite is
green while a whole problem class does not work in the browser. Fix: run the
interactor in the same thread as a coroutine (the interpreter already knows
how to stop at an unanswered `read`), or assert the specific "not supported
in the playground" diagnostic instead of skipping.

---

## 4. Test cheating and hidden failures

### 4.1 P0: the corpus test checks, never runs, and excludes 828 of 1,001 ports

`crates/cpl-cli/tests/corpus.rs`:

- line 37: the only command it runs is `cpl check`;
- lines 46-74: `PENDING`, 27 files that are allowed to fail;
- line 79: `STALE`; line 134: `PROTOTYPE_ONLY`;
- lines 85-88: `proto/ports/scout_batch_*` is left out entirely;
- line 116: passes as long as 80 files were found.

Given 1.1 and 1.2, "checks clean" means little. The real state at
`5879196`:

```
proto/ports (1,001 files):  458 build   233 check-clean but fail build   310 fail check
  top level (76):            23          30                                23
  scout_batch_* (828):      397         171                               260
```

The most common build failures among the 233: "`cpl run` doesn't know the
name/fn" (53), `E0302` type errors the checker missed (44), "no method/member
that `cpl run` knows" (38), "array dimensions that don't start at 0" (12).

Fix: the corpus test runs `cpl build` on every port and, where an input and
expected output exist, runs it. Replace the allowlists with a generated
`research/corpus-status.md` (three counts and the list), committed by the
maintainer pass, so the number is visible and can only go one way. Ports
written in a pre-0.2.2 dialect are migrated or moved to `proto/ports/legacy/`
and not counted.

### 4.2 P0: the published scoreboard is the prototype's, graded by regex

`research/perf/corpus-500-report.md:7-8`: "566 of those 579 problems have a port that
a harness differentially checked against the original". The harnesses are the
Python prototype's, and a pass is a regex match over their logs
(`bench/corpus_registry.py:317-318`):

```python
PASS = re.compile(r"\bOK\b|\bok\b|\bPASS(ED)?\b|\bpass(ed)?\b|agree|✓|\bverified\b|\bvalidated\b", re.I)
```

The same report says 298 files are "spec-form ... which no implementation
accepts yet" (line 13), and its generation split is 530 v1 files, 378 v2,
56 v2.1: almost none of the corpus is 0.2.2. The conciseness headline (65.7%
raw, 47.4% library-stripped) is therefore measured on code the shipping
compiler mostly cannot build, and for 488 of 579 ports the "stripped" side
comes from a name-matching heuristic (`auto_strip`, `corpus_registry.py:33`),
not hand ranges.

Fix: regenerate the registry from the Rust `cpl` with structured results
(exit status and output diff per port, no log scraping), report 0.2.2 ports
separately, and quote only that number.

### 4.3 P0: the fuzzer and audits in `cpl-verify` test the Python prototype

`crates/cpl-verify/src/backend.rs:4-7`: "Today the transpiler is the Python
prototype in `proto/cpc/`". Its only tests are `tests/proto_fuzz.rs` and
`tests/proto_tools.rs`; `src/main.rs:35` defaults to `python3 proto/cp.py
build`. The approved "differential fuzzing of the Rust interpreter against
ASan/UBSan C++17 on every commit" does not exist for the Rust backends. Their
only differential coverage is the 47 fixed programs under
`crates/cpl-cli/tests/programs/` (`tests/differential.rs`).

Also `crates/cpl-verify/src/fuzz/gen.rs:57-58`: "Candidates whose evaluation
traps (overflow, index out of range, too many steps) are skipped". So the
exact class where the backends disagree (section 5) is never generated.

Fix: point `cpl-verify` at `cpl run` and `cpl build` (the
`CommandTranspiler`/`CommandInterpreter` traits already exist), and add a
trap mode: when the reference traps, the interpreter must trap with the same
message and the sanitizer build must not report UB.

### 4.4 P1: there is no CI

No `.github/` directory exists. `cargo test` passes locally (all 50 test
binaries), but nothing runs it, the differential suite or the sweep on a
push, with many sessions pushing to `main` every hour. Fix: one workflow:
`cargo test --workspace`, the port sweep with a "count may not drop" gate,
`cpl test stdlib/*.cp`, and the VS Code and site tests.

### 4.5 P1: stdlib tests run on the interpreter only

`cpl test stdlib/*.cp` is 119/119 on `cpl-interp`. Nothing builds those tests
to C++ and runs them under sanitizers, so the library is unverified on the
backend contestants submit. Fix: `cpl test --release` (or a cargo test) that
does both and compares.

### 4.6 P2: two generations of benchmark harness

`bench/max_n_audit.py` (prototype, 30 ports) sits beside
`bench/rust_max_n_audit.py` (Rust, 40 ports); `bench/README.md:28` documents
both. Delete the prototype one once 3.2 is done.

---

## 5. Runtime and codegen: silent UB in the judge build

`decisions.md` 7.2 settles one thing: unproven **overflow** is "trapped in
local builds, unchecked in the judge build". What the code does goes well
past that. `cpl build` has no local/trapping mode (its only flags are
`-o`), and it drops every check except list bounds:

```
let x: Int = read()            # input 3000000000
print(x * x * x)
  cpl run:        R0001 integer overflow
  cpl build -O2:  -2666827603905609728
  + UBSan:        signed integer overflow

let k: Int = read()            # input 70
print(1 << k, 5 >> k)
  cpl run:        R0001 a shift by 70 is outside 0..64
  cpl build -O2:  64 0

let k: Int = read()            # input 0
print(10 / k)
  cpl run:        R0001 division by zero
  cpl build:      SIGFPE, no message

let n: 1..=10 = read()         # input 1000000 1 2
let a: [n]Int = read()
print(sum(a))
  cpl run:        R0001 1000000 is outside `1..=10`
  cpl build:      3

let x: Int = read()            # input -9223372036854775808
print(abs(x), -x, x / -1)
  cpl run:        R0001 integer overflow
  cpl build -O2:  -9223372036854775808 three times
```

### 5.1 P0: optimizations are proved from checks the judge build does not perform

`crates/cpl-codegen/src/niche.rs:15-17`: the analysis that packs `?Int`
tables with `Int.MIN` as `none` "trusts what `cpl run` checks: declared input
ranges, and that arithmetic does not overflow". `crates/cpl-codegen/src/ranges.rs:295`:
"The interpreter rejects a shift that overflows", used to prove a value
non-negative, which then selects native `/` and `%` in place of the Euclidean
forms. In the emitted C++ none of those checks exist, so one overflow or one
out-of-range input turns into a stored value silently becoming `none`, or a
`%` with the wrong sign. That is "zero-UB compiler trust" (a P0 goal) broken
by construction.

Fix: (a) add the local build: `cpl build --checked` (and `cpl run
--release`) emits every trap the interpreter has, and `tests/differential.rs`
uses it under sanitizers on trap inputs too; (b) in the judge build keep the
cheap checks whose absence is UB rather than a wrong answer: shift range,
division by zero, `Int.MIN / -1`, and input range (one compare per token
read); (c) any optimization fact must be derived only from checks present in
the build being emitted.

### 5.2 P1: `read_int` in the emitted runtime accepts anything

The runtime (`crates/cpl-codegen/src/runtime.rs`, `read_int`) accumulates
digits with no overflow check and reads a non-number (`abc`) as 0; `cpl run`
rejects both (`R0001 expected a number`, `does not fit in an Int`). Covered by 5.1(b).

### 5.3 P2: recursion depth differs

`fn f(n) = if n == 0: 0 else: 1 + f(n - 1)` with `n = 10^7`: `cpl run`
reports runaway recursion (at depth 128,023 in the probe; the constant is
`MAX_DEPTH = 400_000`, `crates/cpl-interp/src/lib.rs:38`), `-O2` C++ returns
10000000, ASan build overflows the stack. A 10^6-node chain DFS is ordinary
contest code; the local run should not refuse what the judge accepts. Fix:
run the interpreter on a large dedicated stack and raise the limit to what an
8 MB / 256 MB judge stack allows, or document the limit and make it a flag.

---

## 6. Dead code, duplication, stale text

- **P1** `crates/prover/` (7,307 lines) and most of `crates/cpl-verify/` are
  unreachable from the product (2.1, 4.3). Wire in or move out of `crates/`.
- **P1** Two JSON implementations (`cpl-lsp/src/json.rs`,
  `cpl-verify/src/json.rs`), two `.cp` expression parsers (`cpl-syntax`,
  `prover/src/parse.rs`), three whole-language parsers counting `proto/cpc`
  and `proto/cpi`.
- **P2** `prelude.rs:63`: `"F64" | "Float"`, "`Float` is still read". A
  pre-0.2.2 spelling kept alive; remove with the port migration.
- **P2** Dead lowering arms for `clz` and `bit` (`call.rs:842-861`): the
  checker rejects both names, so no program reaches them.
- **P2** `crates/cpl-ir/src/lower/mono.rs:88`: the only `#[allow(dead_code)]`
  in the workspace; delete the field or use it.
- **P2** `stdlib/README.md:20-34` describes a world where the Rust backends do
  not exist; `bench/README.md` leads with the prototype harness.
- **P2** `editors/vscode/src/languageData.ts` header promises "zero stale
  v1/v2.1 tokens" while listing 19 names that do not exist (1.5).

What is *not* slop, for the record: the workspace has no `#[ignore]`, no
`todo!()`/`unimplemented!()`, no `TODO`/`FIXME`; `cargo test --workspace` is
green; list indexing and `pop` trap identically on both backends;
`tests/differential.rs` genuinely runs each of its programs under
ASan/UBSan. The problem is not sloppy lines, it is that the green signals
measure the wrong thing.

---

## Cleanup waves

**Wave 1: make the signals honest (no language change, about a day each).**

1. Corpus test builds and runs every port; publish the three counts; delete
   `PENDING`/`STALE`/`PROTOTYPE_ONLY` (4.1).
2. Delete `PROTOTYPE_NAMES`; let the failures show (1.2).
3. Regenerate the registry from the Rust `cpl`, structured results only (4.2).
4. Point `cpl-verify`'s fuzzer at `cpl run` vs `cpl build`, with trap mode
   (4.3).
5. Add CI (4.4). Fix the two stale READMEs (3.2, 6).

**Wave 2: one front end, one prelude.**

6. `cpl-check` hands `cpl-ir` a typed, resolved tree; `cpl-ir` loses its 138
   diagnostics; `cpl check`, the language server, `run` and `build` agree by
   construction (3.1). This closes 1.1, 2.3 (as editor-visible errors), 2.6
   and 3.3's missing diagnostics.
7. `stdlib/prelude.cp` is the only list of auto-imported names; delete the
   four Rust tables and the text-search import; generate the editor's list
   (1.5, 1.6).
8. One decorator table, validated in the checker (2.2).

**Wave 3: move userland out of the compiler.**

9. `Subset<D>` into `stdlib/` on the general index mechanism (1.3).
10. `runs`, `sort`, `sorted`, `shuffle`, `min`/`max`/`sum`/... and the method
    tables into `stdlib/prelude.cp`, leaving a listed intrinsics layer (1.4).
11. Real `U8`/`I32`/`U64`/`I128`, `a.raw[i]`, and a decision on each of the
    29 "can't handle yet" features: implement or delete (2.3-2.5).

**Wave 4: zero-UB judge build.**

12. `cpl build --checked`; differential tests on trap inputs; cheap
    UB-preventing checks kept in the judge build; optimizer facts only from
    checks that exist (5.1, 5.2).
13. `@law` is proved or tested, with the prover on the real AST, or laws are
    declared documentation (2.1).
14. `cpl test --release` for the stdlib (4.5); interactor in the playground
    (3.4); recursion limit (5.3).

**Wave 5: delete.**

15. `proto/cpc`, `proto/cpi`, `bench/max_n_audit.py` leave the gating path;
    duplicate JSON and parser code goes; `Float`, dead arms and stale docs
    go (6).

**Standing rule for the maintainer pass.** After every merge window, rerun
the port sweep and the trap probes in section 5, and reject any commit that
(a) adds a name to a Rust prelude table, (b) adds an `unsupported(` site
without a matching checker error, (c) adds an entry to an allowlist, or
(d) lowers the build count.

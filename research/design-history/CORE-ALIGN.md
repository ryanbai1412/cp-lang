# Core design: problems and proposals [proposed]

This doc is organized by **problem area**, not by feature. Each area has four parts:

1. **Problems:** every job the language has to do in that area, with the exemplar or port that needs it.
   These are numbered (P3.2 = area 3, problem 2) so we can argue about them separately from designs.
2. **Constraints:** what any answer must respect (from ALIGN/GOALS and your calls so far).
3. **Designs:** 3–5 complete answers to the *whole* problem list, each with code and a coverage row:
   ✓ solves it, ~ solves it with a cost, ✗ doesn't.
4. **Pick and open questions:** my recommendation, and what's still unresolved even if you agree.

The test for every area is your four spec goals: **two implementers build the same language**, **almost
everything is userland**, **the core supports everything the library needs**, **a solid library surface**.
Nothing here is decided until you say so.

| # | area | the question | today | pick |
|---|---|---|---|---|
| 1 | Saying things about declarations | how code attaches behaviour, roles, impls and diagnostics to a declaration | `@` with 3 meanings, `#` attributes, markers on "the program" | `@x` = apply library item `x`; compiler things leave `@` |
| 2 | Program shape | tests, per-program vs per-test state, early exit, tools in the same file | file body + `@multitest` + `static` | wrap line `--- with multitest() ---`: rest of file is the per-test closure, flat |
| 3 | Ambient settings | base, formats, checks, lints, stress, library settings | `context` / `with` / `uses` | one program-wide setting per config; call args override |
| 4 | Types carrying runtime data | domains and moduli whose size/value is read at runtime | hidden arguments, partly specified | one slot per declaration site |
| 5 | Integers and index safety | ranges, checks, overflow, widths, literals | interval arithmetic; narrowing unspecified | no flow narrowing; complete rule list |
| 6 | Absence | unvisited, unreachable, fill phases, early exits | lifting, `?`, `??`, `!`, `if let` | drop `?`, lift indexing, add `let … else` |
| 7 | Mutation and effects | in-place helpers, relax-in-condition, purity for lazy/memo | captures only; `if x min= y:`; written `pure` | `var` params + `&`; `chmin`; inferred purity |
| 8 | Stack and layout | deep recursion, inline vs heap arrays | 6 depth shapes, `@depth`, size cutoffs in semantics | big stack, no depth tracking; layout in lowering contract [agreed] |
| 9 | Library machinery | variadics, derive, dispatch, coherence, modules | used, never specified | variadic generics core; derive library; modules need their own doc |
| 10 | One-off syntax | forms that exist exactly once | ~9 such forms | delete or generalize each |

Suggested order to decide: **2 → 1 → 3 → 6 → 7 → 4 → 5 → 8 → 9 → 10**. Area 2 decides what "the program" is,
which removes half of area 1; area 3 depends on whether settings attach to declarations (1) or the program (2).

---

## 1. Saying things about declarations

### Problems
- **P1.1 Wrap a function.** Memoize a recursive function, with recursive calls going through the wrapper
  (Classy Numbers 18, every digit DP).
- **P1.2 Run something per test.** Multitest (Beautiful Graph 24, Cover it! 25).
- **P1.3 Mark code as tooling.** Brute force, generator, checker, interactor, mock: same file (your call),
  stripped from the submission. Must work on **nested** functions too (you flagged that `@alt` only on
  top-level functions felt unnatural).
- **P1.4 Generate impls from a type's shape.** `Read`, `Print`, `Eq`, `Ord`, `Hash`, `Index` for structs and
  enums, including traits from a user library.
- **P1.5 Library-authored diagnostics.** "`Bool` doesn't implement `Print`; use `yes(b)`", written by the
  library that owns `Print` (your answer 5: good errors, maybe via extensions).
- **P1.6 Tag functions for lints.** "Dropping `chmin`'s result is fine" (`@droppable`), "this is hot".
- **P1.7 Tell the compiler or bundler something.** Raw C++, grader entry point, check levels.
- **P1.8 Configure any of the above.** `memo` with an array vs hash table; multitest formats (count first,
  until EOF, "Case #i:" prefixes).

### Constraints
- "Go to definition" works on every annotation; a user library can define new ones (userland).
- One spelling per idea; you can tell from the source whether something is core.
- Tools (stress, bundler, debugger) and the compiler see the same thing.

### Today
| form | what it really is |
|---|---|
| `@memo`, `@droppable` (multitest: area 2) | library function wrapping or tagging a declaration |
| `@alt`, `@gen`, `@check`, `@interactor`, `@mock` | tool role, stripped from the submission |
| `@grader`, `@cpp` (`@depth`, `@decreases` deleted, area 8) | compiler pragmas |
| `#derive(…)`, `#on_unimplemented(…)` | compile-time attributes, a second sigil |
| markers before the first statement | attach to "the program" |

You can't predict what `@x` does without knowing which bucket `x` is in.

### Designs

**A. `@x(args)` applies the library item `x` to the next declaration; compiler things leave `@`.**
```cp
@memo                                 // fn memo<A, R>(f: pure (A) -> R) -> pure (A) -> R
fn ways(i: 0..=n, tight: Bool) -> Z: …

@memo(table: .hash)                   // P1.8: ordinary named arguments
fn grundy(x: Int) -> Int: …

fn solve():
    fn count(l: I, r: I) -> Int: …
    @alt fn count(l: I, r: I) -> Int = (l..r).count(where: ok)   // P1.3: nested alt, same captures

@derive(Read, Print)                  // P1.4: library code over the struct's field tuple (area 9)
struct Edge(u: V, v: V, w: Int)

@on_unimplemented("`{T}` can't be printed; for Bool use yes(b)")   // P1.5
trait Print: …
```
- A **decorator** is a function from declaration to declaration. `@memo fn f` ≡ `let f = memo(body)`, where
  calls to `f` inside the body go to the decorated `f`.
- A **marker** is a decorator that returns its input and registers it with a tool. `alt`, `gen`, `check`,
  `interactor`, `mock`, `droppable` are markers. Tool roles stop being their own kind.
- On types and traits, a decorator receives the declaration's shape (fields, variants) and adds impls.
- Compiler pragmas leave `@`: raw C++ becomes the intrinsic expression `cpp("…")`, checks become a block
  pragma (area 3), depth annotations are deleted (area 8), `@grader` becomes a marker the bundler reads.

**B. Keyword modifiers** (`memo fn f`, `alt fn g`, like Kotlin's `inline fun`). Reads well; a library can't
add keywords, so every modifier is core.

**C. No annotation syntax; ordinary calls.** `let ways = memo(self => (i, tight) => …)`. Fully userland, but
recursion needs an explicit fixpoint, which is noise on the most common use.

**D. Two sigils, Rust's split.** `@x` for library decorators, `#x` for compiler pragmas (`#cpp`, `#checks`).
Honest about what's core, but keeps a second sigil for about three uses.

| | P1.1 | P1.2 | P1.3 | P1.4 | P1.5 | P1.6 | P1.7 | P1.8 | userland | one spelling |
|---|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ via intrinsics/pragmas | ✓ | ✓ | ✓ |
| B | ✓ | ✓ | ✓ | ~ needs `derive` keyword | ✗ | ~ | ✓ | ~ | ✗ | ✓ |
| C | ~ fixpoint | ✓ `multitest()` | ~ `alt(count, brute)` | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ |
| D | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ~ two sigils |

**Pick A.**

**Open:** the exact decorator type on a nested function (it can capture); what shape a type decorator sees
(area 9); whether decorators may change a function's signature (I'd say no: `memo` keeps it).

---

## 2. Program shape

### Problems
- **P2.1 Zero ceremony for single-test problems.** Most exemplars are just statements.
- **P2.2 Multitest in all its formats.** Count first; count on the same line; until EOF; "Case #i: " prefixes.
- **P2.3 Per-program state.** Factorials or a sieve computed once and visible to every test.
- **P2.4 Per-test state.** Reset every test; fresh types (domains) die with the test (ALIGN H3).
- **P2.5 Early exit from one test.** Print `-1`, go to the next test, without ending the program.
- **P2.6 Tools next to the solution** (P1.3), including stress-testing a nested chunk.
- **P2.7 Interactive problems** in the same shape, with the solution and interactor in one file.
- **P2.8 Grader problems** (IOI-style "implement `fn f`"), where there is no `main`.
- **P2.9 Long solutions split into chunks:** nested functions sharing captures (your decomposition goal).

### Constraints
- No rule that applies only to "the file body".
- The same code works when pasted into a function (graceful cold path).

### Today
The file body is the program; markers before the first statement decorate it; `static let` opts out of
per-test state; `return` at file level ends the current test. Four special rules, and `@multitest`
decorates something that isn't a declaration.

### The tension
Helpers need per-test state (`V`, `adj`, `depth`) without threading it through parameters, and a
300-line solution must not end up indented inside one function (your 2026-11-21 note). Nested functions
solve the first and fail the second.

### Designs (nothing nested in any of them)

**A. The file body is `solve`; `main` calls it.**
```cp
with io(base: 1)

fn main() = multitest(solve)                 // omit for one test: main defaults to solve()

once let fact = Z.factorials(2e5)            // P2.3

domain V = read(5..=1e5)                     // solve's body: per test, flat (P2.4)
let adj = adjacency(read<[read()](V, V)>())
var depth: [V]?Int = none

fn long_cycle(v: V, d: Int) -> ?List<V>:    // sees V, adj, depth (P2.9)
    …

if let c = long_cycle(V.first, 0):
    print(c)
    return                                   // returns from solve (P2.5)
…
```
- Top-level functions are nested functions of `solve`, unindented. They are hoisted, and a call before a
  captured binding is initialized is a compile error.
- Multitest formats are ordinary code (P2.2): `multitest(solve, count: .eof)`,
  `multitest(solve, prefix: i => "Case #{i}: ")`, or any loop in `main`.
- Special rules: the file body is `solve`; `main` is outside it; `once`.

**B. A with an explicit boundary line.** Shared helpers and `main` above a `solve:` line, the per-test
body below it, still unindented. More honest about where per-test state starts; costs a section keyword.

**C. Explicit `fn main()` and `fn solve()`, per-test state in reset globals.**
```cp
fn main() = multitest(solve)
fn solve():
    V = read(5..=1e5)
    print(long_cycle(V.first, 0) ?? independent_set())

test domain V
test var depth: [V]?Int = none
fn long_cycle(v: V, d: Int) -> ?List<V>: …
```
Familiar from C++ globals, with automatic reset. Each binding is declared in one place and filled in
another, and a late-bound domain is awkward.

**D. A named state struct with outside methods.** `struct Test(domain V, adj: [V]List<V>, …)` and
`fn Test.long_cycle(…)` read fields unqualified; `main` calls `Test.read().solve()`. Chunk boundaries are
written down and a stress test can build a `Test`; every field is written twice.

**E. Pure chunks.** Generic top-level functions; data flows through arguments and returns. Zero special
rules; long parameter lists.

**F. Explicit `fn solve():` with nested chunks** (the previous pick). One indentation level over the whole
solution.

**G. A wrap line passes the rest of the file to a function** (Gleam `use`, Koka `with`, Typst `#show:`).
```cp
with io(base: 1)
let fact = Z.factorials(2e5)                 // above the line: per program (P2.3)

@!multitest                     // the rest of the file is the per-test closure (P2.2)
domain V = read(5..=1e5)                     // per test, flat (P2.4)
let adj = adjacency(read<[m](V, V)>())
var depth: [V]?Int = none

fn long_cycle(v: V, d: Int) -> ?List<V>:    // sees V, adj, depth (P2.9)
    …

if let c = long_cycle(V.first, 0):
    print(c)
    return                                   // returns from the closure: the next test (P2.5)
…
```
- One rule, sugar: `--- with f(a) ---` + rest `R` ≡ `f(a, () => R)`; `--- with x <- f(a) ---` ≡ `f(a, x => R)`.
- No special names (`solve`, `main`), no keyword for per-program state, and it works the same in a
  library (top-level bindings there are per program, built on first use).
- Formats are functions (P2.2): `multitest(count: .eof)`, `multitest(prefix: i => "Case #{i}: ")`,
  `case <- tests()`, or your own `fn pairs(body: (Int, Int) => ())`.
- Brute, gen and checker are ordinary functions above the line, so they can't see per-test state (P2.6);
  `@alt(program)` replaces the closure.

| | P2.1 | P2.2 | P2.3 | P2.4 | P2.5 | P2.6 | P2.7 | P2.8 | P2.9 flat | special rules |
|---|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ library | ✓ `once` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 3 |
| B | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 2 + a section keyword |
| C | ~ | ✓ | ✓ | ~ late binding | ✓ | ✓ | ✓ | ✓ | ✓ | 2 |
| D | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ~ fields twice | 1 |
| E | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ parameters | 0 |
| F | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ indented | 1 |
| G | ✓ | ✓ library | ✓ by position | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 1 (the wrap line, sugar) |

Also considered during the 2026-11-21 discussion: named sections (`--- solve`, `--- brute`; rejected
because brute and gen aren't special), per-program state as `main`'s locals (no `main` in a library), and
other spellings of the wrap (`multitest(() => ...)` collides with the docs' elision mark; `<- multitest()`
is cryptic; a bare `with multitest()` line doesn't look like a boundary).

**Deferred** [you, 2026-11-21: not top priority; the rest of the language shouldn't depend on it]. **Pick G** [applied provisionally, spec 16.3]. Chosen with you on 2026-11-21. A single-test file is just
statements, per-program vs per-test is decided by which side of one visible line a binding is on, and
multitest is a library function you can replace.

**Open:** whether gens produce `Case` values or text; whether `with ctx(…)` contexts become `set` defaults
so `with` only means wrapping (area 3).

---

## 3. Ambient settings

### Problems
- **P3.1 Input/output base.** 1-based vertices in, 1-based out, for every domain value, without touching each
  read (the original `@base(1)` ask, generalized at your request to `io(base: 1)`).
- **P3.2 The cold path.** One 0-based quantity in a 1-based file, as a one-token local edit (goal I.15).
- **P3.3 Output format.** Separator, `YES`/`Yes`, float precision.
- **P3.4 Checks.** Turn bounds/overflow checks off in one hot loop; force them on somewhere in release.
- **P3.5 Lints.** Float-equality lint on or off per file or per site (your answer 6).
- **P3.6 Stress settings.** Test count, seed, sizes.
- **P3.7 Library settings.** A hashing library's base, a geometry `eps`, declared by the library.
- **P3.8 Compile-time.** Settings that change codegen cost nothing at runtime.
- **P3.9 Agreement.** The interactor and solution in one file use the same format.

### Constraints
- No runtime values carried implicitly. Round 5 rejected that after a table built under one modulus was read
  under another.
- The library declares every setting; the compiler knows only the ones it really consumes.

### Today
`context` declarations; `with` at file or block level; `uses` to follow the caller's setting; inference for
nested functions only; one specialized copy per value. A whole mechanism for four contexts (`io`, `checks`,
`lint`, `stress`), and the compiler reads `checks`/`lint` itself, so they're special anyway.

### Designs

**A. Program-wide configs; call arguments override.**
```cp
config io:                             // declared by the I/O library: compile-time fields with defaults
    base: 0..=1 = 0
    sep: Str = " "

with io(base: 1)                       // set once, at the top of the file

impl<D: Counted> Read for D:
    fn read(base: 0..=1 = io.base) -> D = D.at(read<Int>() - base)

let k: V = read(base: 0)               // P3.2: the cold path is a call argument

with checks(bounds: off):              // P3.4/P3.5: checks and lints are core pragmas, block-scoped
    for i in 0..n: …
```
- One value per program, so there's no lexical-vs-dynamic question, no `uses`, no inference, no clones.
- `checks` and `lint` are listed as core, which they are.
- Lost: setting `io` differently in one block. No port has wanted it.

**B. Today's contexts, fully specified.** Lexical scoping, `uses`, inference and cloning rules all written out.
The most power; the most spec text; the most places for two implementers to disagree.

**C. Implicit parameters (Scala `given`).** Resolved by type at each call. Brings back runtime values carried
implicitly (rejected in round 5).

**D. Explicit config values.** `let cfg = Io(base: 1); read(cfg)`. No mechanism, but every call carries it.

**E. Parameterized imports.** `use io(base: 1)` instantiates the I/O module with constants (ML functors).
No new mechanism once modules exist, and library settings (P3.7) come free; but the module system isn't
designed yet (area 9), and checks/lints still need block scope.

| | P3.1 | P3.2 | P3.3 | P3.4 | P3.5 | P3.6 | P3.7 | P3.8 | P3.9 | mechanism size |
|---|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ pragma | ✓ pragma | ✓ | ✓ | ✓ | ✓ | small |
| B | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | large |
| C | ✓ | ✓ | ✓ | ~ | ~ | ✓ | ✓ | ✗ | ✓ | medium |
| D | ~ | ✓ | ~ | ✗ | ✗ | ✓ | ✓ | ✓ | ~ | none |
| E | ✓ | ✓ | ✓ | ~ needs pragma | ~ | ✓ | ✓ | ✓ | ✓ | none beyond modules |

**Deferred** with area 2 [you, 2026-11-21]; the `#set` proposal (Typst-style named-default changes) is the leading candidate. **Pick A, spelled so it can become E** once modules exist (`with io(…)` at the top of a file is an import
parameter in all but name).

**Open:** whether `config` fields may be types (e.g. a default `Mint` modulus); whether a setting can be
read in a type (`[io.base..]T`) — I'd say no.

---

## 4. Types carrying runtime data

### Problems
- **P4.1 Nominal indices.** Arrays indexed by a vertex type whose count is read at runtime; mixing vertex and
  edge indices is a compile error.
- **P4.2 Per-test freshness.** A `V` from one test can't be used in the next (ALIGN H3).
- **P4.3 Userland domains.** Counted, `Cut`, `grid`, products, `labels`, `ring` are library code over the core.
- **P4.4 Derived domains from functions.** `components`, `compress`, `Piece` are returned as new types.
- **P4.5 Runtime moduli.** `Mint(mod: p)` with `p` from input; two moduli at once (double hashing).
- **P4.6 Generic code without plumbing.** A segment tree over `V`, `bfs` over `V`, and `Mint` arithmetic work
  without passing `n` or `p` around.
- **P4.7 Captured and stored.** Closures and structs hold `V` values and `[V]T` arrays.
- **P4.8 Recursive declarations.** A recursive function declares a domain inside (divide and conquer,
  centroid decomposition per level).
- **P4.9 Zero overhead.** `V` is a 32-bit int; hot loops do no extra work beyond C++.
- **P4.10 Exact rules.** Type equality between fresh types, and every form of escape, are written down.

### Constraints
- This is the one novel mechanism; it has to be the best-specified part of the spec, not the least.

### Today
Generic code gets the data as a hidden argument "per activation", or from a static "when no reentry is
possible". The spec doesn't say how closures or generic structs carry it, or what "reentry" means.

### Designs

**A. One slot per declaration site, saved and restored on re-entry.**
```cp
fn solve():
    domain V = read(1..=2e5)           // writes V's slot: count = n
    let tree = SegTree<V, Max>()       // instantiated at V; reads V's slot for its size
    let (domain P, piece) = g.components()   // the callee returns P's data; this binding owns P's slot
```
- Each `domain`/`type` declaration owns one static slot. Binding writes it. Leaving the block restores the old
  value, which only matters when the declaration can be re-entered recursively (P4.8).
- **Why it's sound:** a value whose type mentions a fresh type can't leave its block, so while such a value
  exists, its block is active and the slot is current. A recursive callee can't receive the outer `V`'s values,
  because its parameter types can't name a domain declared in its own body.
- Generic code is monomorphized per declaration site, so closures and structs need nothing extra.
- Cost: one store per binding, one save/restore per recursive re-entry, one global load where used.

**B. Hidden arguments (dictionary passing), fully specified.** Generic functions take the data as extra
parameters; closures capture it; generic structs store it. Classic and well understood, but it's more spec
text, and the "static when no reentry" optimization is design A anyway.

**C. Explicit witnesses.** `let V = Domain(n)` is a value whose type is fresh (Rust's `generativity` trick);
generic functions take `V` explicitly. No hidden anything; every generic call passes the witness by hand.

**D. Static slots, no re-entry.** Like A but declaring a domain inside a recursive function is an error.
Simplest of all; fails P4.8.

| | P4.1 | P4.2 | P4.3 | P4.4 | P4.5 | P4.6 | P4.7 | P4.8 | P4.9 | P4.10 |
|---|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ short |
| B | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ~ capture rules | ✓ | ~ extra params | ~ long |
| C | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ |
| D | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ shortest |

**Pick A** [agreed with you 2026-11-21; written into spec 12.2–12.3, including the escape list].

**Was open:** the full escape list (return; store into an outer `var`; a closure that escapes; a top-level
binding; across `solve` calls) and whether a domain can be a struct field's type outside its block
(I'd say no, same rule).

---

## 5. Integers and index safety

### Problems
- **P5.1 Off-by-one-free indexing.** Most indices come from domains, so most indexing needs no check (P0).
- **P5.2 Input bounds from the statement.** `read(1..=2e5)` catches wrong assumptions in debug.
- **P5.3 Index arithmetic.** `i + 1`, `mid`, positions vs cuts, distances between cuts.
- **P5.4 Check elimination.** Know statically when `a[i]` is in range.
- **P5.5 Overflow.** Wrap in release, trap in debug (your answer 2).
- **P5.6 Fixed widths.** `I32`/`U64` for memory, explicit conversions.
- **P5.7 Predictable types.** Two implementers infer the same range for every expression.
- **P5.8 Literals.** `2e5` is an `Int`.
- **P5.9 Optional `Int` costs nothing** (niche).

### Constraints
- The type of an expression shouldn't depend on reasoning the reader can't see.

### Designs

**A. No flow narrowing; a complete list of where ranges come from.**
- A range comes from an annotation, a read, a domain's `count`, a conversion, or interval arithmetic over
  `+ - * / % min max abs`, with the hull rule for `if` and list literals. That's the whole list.
- `let` keeps the computed range; an unannotated `var` widens to `Int`; `var x: lo..=hi` checks each assignment.
- Narrowing is explicit: `(0..n).check(x)` traps, `(0..n).try(x)` returns `?T`.

**B. Narrowing on comparisons (TypeScript style).** Inside `if x < n:`, `x: ..n`. Proves more indexing safe;
the spec must list which conditions narrow (`and`, `not`, early `return`, loops, writes in between).

**C. Ranges only at boundaries.** Fields, parameters and reads have ranges; locals are plain `Int`.
Simplest; loses `x % d` indexing a `[0..d]T` with no check.

**D. Full refinement types** (Liquid Haskell). Proves the most; needs an SMT solver, and error messages are
hard. Rejected.

| | P5.1 | P5.2 | P5.3 | P5.4 | P5.5 | P5.6 | P5.7 | P5.8 | P5.9 |
|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ~ checks stay where flow would prove them | ✓ | ✓ | ✓ | ✓ | ✓ |
| B | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ~ edge rules | ✓ | ✓ |
| C | ~ | ✓ | ~ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ |
| D | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ |

**Pick A** [agreed with you 2026-11-21]. The optimizer can still remove checks that flow proves; that's lowering, not typing.

**Settled (spec 11.3–11.4 already matched):** floor `/` and `%`; `Int` excludes `INT64_MIN`, and release wrap onto it is documented. **Was open:** the result range of `/` and `%` with negative operands; whether `Int`'s niche (`INT64_MIN`) can be
reached by release wrap-around and read as `none` (I'd document it; debug traps first).

---

## 6. Absence

### Problems
- **P6.1 Unvisited markers** in `dist`/`parent` arrays without `-1` sentinels.
- **P6.2 Fill phase, then all-present phase.** `let depth = depth!` (Cover it! 25).
- **P6.3 Unreachable DP states.** `dp[b] max= dp[j] + w`, where `none` propagates (your call: `none + x = none`).
- **P6.4 Defaults,** including diverging ones: `?? 0`, `?? return`, `?? break`.
- **P6.5 Early exit with several statements.** Print, then return (`let … else`).
- **P6.6 Loop while present.** `while let v = q.pop_front()`.
- **P6.7 Patterns that aren't `?T`.** Pull `v` out of `ask(v)`.
- **P6.8 Optional index or field.** `cost[parent[v]]`, the length of an optional string.
- **P6.9 Zero-cost layout** through niches.
- **P6.10 Absence stays visible.** Ordering never lifts, so `none` can't silently win a comparison.

### Designs

**A. Lift indexing like arithmetic; drop postfix `?`; add `let … else`.**
```cp
dp[b] max= dp[j] + w                   // + lifts (unchanged)
let c = cost[parent[v]]                // indexing lifts: c: ?Int
let len = s.map(x => x.len)            // fields: ordinary map
let order = g.topo_order() else:       // the block must diverge
    print("NO")
    return
let ask(v) = q else: fail("expected a query")
```
Indexing lifts through a blanket impl, the same mechanism as `+`. `??`, `!`, `if let`, `while let` stay.

**B. Keep postfix `?` with a simpler extent:** "the whole statement, or the left side of `??`".

**C. Today's set plus `let … else`.** The most expressive, the most to specify.

**D. No lifting (Rust style).** Everything through `map`/`and_then`/`?`. You already chose lifting for
arithmetic; listed for reference.

| | P6.1 | P6.2 | P6.3 | P6.4 | P6.5 | P6.6 | P6.7 | P6.8 | P6.9 | P6.10 | core rules |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | `??`, `if/while let`, `let…else` |
| B | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✓ | + `?` extent |
| C | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | + `?` extent |
| D | ✓ | ✓ | ~ noisy | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | ✓ | `?` |

**Pick A.**

**Open:** whether `let … else` also accepts boolean conditions (`let x = y if c else: …`) — I'd say no.

---

## 7. Mutation and effects

### Problems
- **P7.1 In-place helpers anywhere.** A top-level function mutates the caller's array (unspecified today).
- **P7.2 Nested helpers mutate enclosing state** (decomposition into chunks; works today).
- **P7.3 Relax in a condition.** `if dist[v] min= d: queue.push(v)` in Dijkstra and BFS.
- **P7.4 No aliasing surprises.** Value semantics; copies are visible.
- **P7.5 Big values are cheap to move and return.**
- **P7.6 Invalidation.** Pushing to a list while iterating it is caught.
- **P7.7 Lazy views and memo.** Their lambdas can't have hidden effects, so evaluation order doesn't matter.
- **P7.8 Debug-sampled checks** (e.g. `partition_point` sampling monotonicity) need pure predicates.

### Designs (three sub-questions; each row is a full set)

**A. `var` parameters with `&` at the call; `chmin` method; inferred purity.**
```cp
fn relax(var dist: [V]?Int, v: V, d: Int) -> Bool = dist[v].chmin(d)
if relax(&dist, w, d + 1): queue.push_back(w)       // P7.1, P7.3
if dist[w].chmin(d + 1): queue.push_back(w)         // P7.3 inline
```
- Swift's `inout`: by reference, marked at both ends. Two `&` to overlapping places in one call is a compile
  error when syntactically identical, a debug trap otherwise.
- `chmin`/`chmax` are `var self` methods (the same mechanism `push` uses) returning whether the value changed;
  `x min= y` is statement sugar for `x.chmin(y)`.
- Purity is inferred, never written: users never type `pure`. It appears only as a bound in library
  signatures (`fn memo<A, R>(f: pure (A) -> R)`). `memo` uses a trusted intrinsic for its table, so "calls
  to `@memo` functions count as pure" stops being an exception.

**B. Keep today's surface, specify it.** Nested functions only for mutation; `if x min= y:` as a one-off
condition form; written `pure` on function types.

**C. Assignments as expressions everywhere (C style).** `if (dist[v] min= d)` and `while (x = f()) …` are
legal anywhere. Removes the one-off by generalizing it; brings back `if x = y` bugs.

**D. Functional style.** No out-parameters (`dist = relax(dist, v, d)`); no purity tracking; lazy adapters
document their order instead. Smallest core; fights in-place CP code.

| | P7.1 | P7.2 | P7.3 | P7.4 | P7.5 | P7.6 | P7.7 | P7.8 | one-offs |
|---|---|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 0 |
| B | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 2 |
| C | ✗ | ✓ | ✓ | ~ | ✓ | ✓ | ✓ | ✓ | 0, but a bug class |
| D | ~ noisy | ✓ | ~ | ✓ | ✓ | ✓ | ✗ | ✗ | 0 |

**Pick A.**

**Open:** whether `&` is required when passing a nested function's own capture (I'd say yes, for uniformity).

---

## 8. Stack and layout

### Problems
- **P8.1 Deep recursion works.** A DFS 10⁶ deep mustn't crash on a judge whose default stack is small.
- **P8.2 Overflow is diagnosed clearly** in debug, with the function and depth.
- **P8.3 Small fixed arrays are fast** (`[26]Int`, `[4]Dir`), like C arrays.
- **P8.4 Large arrays never land on the stack.**
- **P8.5 Layout never changes meaning,** so two implementers may differ in layout but not in behaviour.
- **P8.6 Layout is visible** (`cp explain`).
- **P8.7 No annotations in ordinary code** (the `@depth(V.count)` noise you asked about).

### Today
Six depth-inference shapes, `@depth`, `@decreases`; "inline if static and ≤ 4 KB", "a read array with literal
`len:` ≤ 8 is inline", and stack-budget rules, all written as language semantics.

### Designs

**A. Big stack always; debug traps; layout moves to the lowering contract.**
- The bundler runs `main` on a thread with a large stack wherever the judge profile allows; a profile without
  threads says so and gets a warning.
- Debug counts depth and traps with "`dfs` depth 3,100,000 × ~96 B exceeds the stack", plus the trace.
- Semantics: arrays are values; copies and moves are as in chapter 16. Lowering contract: inline when static,
  ≤ 4 KB, in a non-recursive function; heap otherwise. `cp explain` shows which.

**B. A + optional `@depth(expr)`** as a checked assertion for documentation.

**C. Today's inference plus annotations.** Every new recursion idiom needs a shape or an annotation.

**D. Automatic heap stacks.** The compiler turns recursion into an explicit heap stack. Never overflows;
slower, and the C++ output gets hard to read.

| | P8.1 | P8.2 | P8.3 | P8.4 | P8.5 | P8.6 | P8.7 |
|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| B | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ~ optional |
| C | ✓ | ~ | ✓ | ✓ | ✗ | ✓ | ✗ |
| D | ✓ | ✓ | ✓ | ✓ | ✓ | ~ | ✓ |

**Pick A, minus the debug depth counter** [agreed with you 2026-11-21: "always assume the judge gives us a big
stack … we shouldn't really do depth tracking"]. Stack size is a judge-profile setting. Written into spec 18.3.

---

## 9. Library machinery

### Problems
- **P9.1 Variadic calls.** `print(a, b, c)` and `read<(V, V, Int)>()` with mixed types.
- **P9.2 Tuples of any arity** implement `Print`, `Eq`, `Ord`, `Hash`, `Read`.
- **P9.3 Impls from a type's shape** (P1.4), including for traits from user libraries.
- **P9.4 Operators are trait calls;** lifting over `?T` is impls.
- **P9.5 Return-type dispatch.** `read<T>()`, `let x: V = read()`.
- **P9.6 Library-authored errors** (P1.5).
- **P9.7 Generic code checked at definition** with good errors (your answer 5).
- **P9.8 Coherence.** Who may implement what for whom (orphan rule), and how methods resolve.
- **P9.9 Modules and versions.** Imports, your local library, pinning versions; the bundler inlines it all.
- **P9.10 Messages over protocols** (your "could just be a macro in userland" for interaction).

### Designs

**A. Variadic generics are core; structs are named tuples; derive is library.**
- Core: variadic type parameters (`Ts...`) and impls over tuples of any arity.
- A struct is a nominal wrapper over its field tuple; an enum over a variant sum. So `@derive(Print)` is
  `impl Print for S: fn print(self, out) = self.fields.print(out)`, ordinary code.

**B. A fixed core derive list** (`Eq`, `Ord`, `Hash`, `Index`, `Read`, `Print`, `Gen`). Simple and honest; user
libraries can't derive their own traits.

**C. Compile-time reflection (Zig `comptime`).** The most general; a large new mechanism.

**D. Hygienic userland macros.** Covers derive and protocol messages (P9.10); macros are hard to type-check at
definition (P9.7) and hard for tools to show.

| | P9.1 | P9.2 | P9.3 | P9.6 | P9.7 | P9.10 | core size |
|---|---|---|---|---|---|---|---|
| A | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ via enums + derive | small |
| B | ✓ | ✓ | ~ | ✓ | ✓ | ~ | smallest |
| C | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | large |
| D | ✓ | ✓ | ✓ | ~ | ✗ | ✓ | medium |

**Pick A.** P9.4, P9.5 and P9.8 are the same in every design and need writing out, not choosing. P9.9
(modules) is missing from the spec entirely and needs its own problems-and-proposals doc.

---

## 10. One-off syntax

Forms that exist exactly once, and what each design above does to them:

| form | where else it appears | fate under the picks |
|---|---|---|
| `if x min= y:` | nowhere | `x.chmin(y)` (area 7) |
| postfix `?` and its extent rule | nowhere | deleted (area 6) |
| markers attached to "the program" | nowhere | deleted (area 2) |
| `static let` | nowhere | deleted: bindings above the wrap line are per program (area 2) |
| `return` ends the test at file level | nowhere | ordinary `return` from the wrap closure (area 2) |
| bare `with` only at a block's top | nowhere | `with` only at file top; pragmas use blocks (area 3) |
| `#` attributes | nowhere | become `@` decorators (area 1) |
| `@depth`, `@decreases` | nowhere | deleted (area 8) |
| enum shorthand `order: desc` beside `.desc` | two spellings | one: leading `.` where the type is known |
| `max=` defined two ways (00 vs 14.3) | contradiction | one: `x max= y` ≡ `x.chmax(y)` |

---

## How the picks fit together

Beautiful Graph (24) written under every pick, as a check that they compose. Illustrative and unrun.

```cp
with io(base: 1)

type Z = Mint(mod: 998244353)
let pow2 = Z(2).powers(3e5 + 1)                  // per program

@!multitest                         // area 2: the rest is the per-test closure

domain V = read(1..=3e5)                         // slot per declaration (area 4)
let m: 0..=3e5 = read()
let g = Graph<V>.undirected(read<[m](V, V)>())   // variadic read of tuples (area 9)
var answer = Z(1)
for comp in g.components():
    let (a, b) = comp.two_coloring() else:       // let … else (area 6)
        print(0)
        return                                   // returns from the closure: the next test
    answer *= pow2[a.len] + pow2[b.len]
print(answer)
```

No `@`, no `static`, no `uses`, no `?`, no depth annotations.

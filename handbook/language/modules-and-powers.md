# Modules and library powers

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/11-modules-prelude.md`, `spec/core/12-powers.md` |
| Describes | `spec/core/11-modules-prelude.md`, `spec/core/12-powers.md`, `std/prelude.cp`, `spec/lib/rationale/`, `spec/conformance/core-11/`, `spec/conformance/core-12/` |

How code is split into files and modules, which names a program sees
without asking, and the mechanisms (powers) that let a library, the
stdlib or a contestant's own, feel built in. What the stdlib contains is
[std.md](std.md); the whole language is [overview.md](overview.md).
Terms are in `spec/GLOSSARY.md`.

## The problem

Every experienced contestant has a template: a few hundred lines of
segment tree, DSU, modular integer and fast I/O pasted above `main`. In
C++ the template works because the language gives a library the same
tools as the standard library: operator overloading, templates,
`operator[]`. Many friendlier languages don't. Python's `sorted` is fast
because it is C; the contestant's own segment tree in Python is 50 times
slower than the same code would be if it were built in. Other languages
make the standard library's collections printable, indexable or
iterable through mechanisms user code can't reach.

The cost shows up in the contest. A contestant writes a "best value and
its position" operation for a segment tree:

```cpp
struct Best { long long v; int at; };
Best pick(Best x, Best y) { return y.v > x.v ? y : x; }
// then: copy the template, change three typedefs, write the identity
// in the one place the template expects it, and hope it matches.
```

If the standard structure takes only built-in operations, the template
gets pasted after all, and the language's safety and speed stop at the
template's edge.

The second problem is names. A language with a big standard library must
choose between `use` lines at the top of every solution and a prelude so
large that it collides with `n`, `g` or `dp`, and the choice has to be
guessable: a contestant must know, without looking, whether `dijkstra`
needs an import.

## The model

### A program is one file; a library is items

```cp
// output: "3 7"
mod stats:
    pub struct Tally(pub n: Int, total: Int):
        pub fn add(mut self, x: Int):
            n += 1
            total += x
        pub fn mean(self) -> Int = total / n
    pub fn tally(xs: List<Int>) -> Tally:
        var t = Tally(0, 0)
        for x in xs: t.add(x)
        t
let t = stats.tally([4, 8, 9])
print(t.n, t.mean())
```

- A **program** file is statements run top to bottom (chapter 10). A
  **library** file holds only items (`fn`, `struct`, `enum`, `trait`,
  `impl`, `type`, `const`, `context`, ...) and `use` lines.
- `pub` marks what a module exports. Fields and methods follow the same
  rule: outside `stats`, `t.total` is an error (E0207) whose fix adds
  `pub`. Inside one file nothing needs `pub`, so a single-file solution
  never writes it.
- An inline `mod name:` groups items inside any file, for a part to
  stress on its own or a grader entry.
- `use std.math.gcd` names the pinned stdlib; `use lib.treap` names the
  contestant's own library roots from `cp.toml`. Modules form an
  acyclic graph.
- Everything is inlined into the one submitted C++ file and tree-shaken,
  and library generics are specialized where they are used, so library
  code is as fast as code in the file.

### The prelude is the documented vocabulary

```cp
// output: "12"
domain V = 1..=3
let g = Graph.undirected([(u: V(1), v: V(2), w: 5), (u: V(2), v: V(3), w: 7)])
let p = dijkstra(g, from: [(V.first!, 0)])
print(p[V.last!] ?? -1)
```

`Graph` and `dijkstra` need no `use`, and the rule for knowing that is
short: **a std name is in the prelude exactly when a `lib/` chapter's
main tables list it**, together with **every std type a prelude
signature names** (parameters, results, getters, yields). So any value a
program can get, it can also annotate: 
`a.ring.minimal` yields
`RingSlice`s, so a lambda over its results can write `(w: RingSlice<M,
Int>) => w.len` with no `use`, even though no chapter table lists
`RingSlice` on its own row (`spec/conformance/core-11/prelude-exports-signature-types.cp`).

Two exclusions keep the prelude from colliding with code:

- A name contestants commonly give a local (`n`, `a`, `g`, `dp`, `ans`,
  `cnt`, `left`, `best`) is never a prelude name.
- A name with two plausible meanings gets one, and the others are
  qualified. `min` and `max` are the builtins over two or more values;
  the folds of one sequence are methods (`xs.max()`).

And a local always wins over the prelude:

```cp
// output: "7"
fn gcd(a: Int, b: Int) -> Int = 7
print(gcd(4, 6))
```

Methods need no import at all: `xs.sorted()` and `g.adjacent(v)` are
found on the receiver's type wherever the type came from. Only free
names need the prelude or a `use`. The prelude itself is one ordinary
file, `std/prelude.cp`, made of `use` lines, and
`scripts/spec-stub.py --check` keeps it in step with the chapter tables.
Library modules don't get the prelude: they `use` what they need, so a
library never depends on what a program happened to import. An inline
`mod` inside a program is part of the program and does see it.

### No stdlib privilege

Being in the prelude is a naming fact only. The stdlib is ordinary CP
Lang in `std/`, and it has **no power a user library lacks**: anything a
stub in `std/` needs is a power listed in `spec/core/12-powers.md`, open
to every library, or it is core syntax. The segment-tree operation from
the problem is two lines:

```cp
// output: "9 1"
struct Best(v: Int, at: Int)
let pick = (x: Best, y: Best) => if y.v > x.v: y else: x
let a = [3, 9, 4, 9]
let st = SegTree(Monoid(Best(-1, -1), pick), [Best(a[i], i) for i in 0..a.len])
let b = st.fold(0..4)
print(b.v, b.at)
```

`SegTree(+, a)` and `SegTree(max, a)` take a bare operator, whose
identity the element type supplies; anything else is `Monoid(id, op)`.
The same tree serves both: the operation is a value, not a new type.

### Powers

A **power** is a mechanism that lets library code feel built in. Each one
earns its place by the stdlib families it unlocks. Three of them, used
from a user type:

A type with `impl Seq` supplies one generator, `iter`, and gets every
generic fold, exactly as std's containers do (R12.4.1):

```cp
// output: "4 2 10"
struct Countdown(from: Int)
impl Seq<Int> for Countdown:
    fn iter(self) -> yields Int:
        var i = self.from
        while i > 0:
            yield i
            i -= 1
let c = Countdown(4)
print(c.iter().max()!, c.iter().count(x => x > 2), c.iter().sum())
```

By R12.4.1 the value itself also walks in `for` and takes the folds
directly (`c.sum()`, `for x in c`); the current `cpl` still needs the
explicit `.iter()`.

A type with `at` and `set` is a place: `x[k] = v` calls `set`, and
`x[k] op= v` is `x.set(k, x[k] op v)`, exactly as for std's `Deque` and
`SegTree`:

```cp
// output: "7 4"
struct Box(xs: List<Int>):
    pub fn at(self, i: Int) -> Int = xs[i]
    pub fn set(mut self, i: Int, v: Int):
        xs[i] = v
var b = Box([1, 2])
b[1] = 7
b[0] += 3
print(b[1], b[0])
```

A row bound takes any record with at least the named fields, which is
how every graph algorithm accepts any edge record:

```cp
// output: "3"
fn f<R: (u: Int, ..)>(x: R) -> Int = x.u
print(f((u: 3, w: 4)))
```

The full list, each with what needs it:

| Power | What a library gets | Stdlib families it unlocks |
|---|---|---|
| W1 numeric types | operators, `Int` literal conversion, `read`/`print` | `Mod<P>`, `Frac`, `Matrix`, `Poly`, `Point` |
| W2 niches | `?T` free when `T` has an unused value | `[V]?Int`, `?Edge` |
| W3 reflection and derives | `==`, `<`, hash, show by field order; `for f in fields(x)` | every record |
| W4 sequences and index spaces | the six capabilities, views, domains | every container |
| W5 patterns | closed: views are methods returning an enum | `t.node(v)` |
| W6 generators and steering | `yields`, `break` through recursive generators | `bfs`, `dfs`, `divisors`, `submasks` |
| W7 functions as values | `SegTree(+)`, `by: <`, named and default arguments | every option |
| W8 user places | `IndexMut` (`at_mut`) or `IndexSet` (`set`) | `Map`, `SegTree`, `Fenwick` |
| W9 operators | `fn (a: T) + (b: T) -> T` | numeric types |
| W10 protocols and laws | traits, `law`, operation values | range structures, `Ord`, `Hash` |
| W10a records and associated types | `G.Arc`, trait aliases, field traits, row bounds | graphs |
| W11 reading and printing | `Read`, `Show` | input shapes |
| W12 costs and library lints | `/// cost:`, `#lint`, `#on_unimplemented` | every signature |
| W13 modules and pins | `use`, `#deprecated(use: ...)`, codemods | migrations |
| W14 inline C++ | `@cpp("...")` with a checked signature | intrinsics |
| W15 precomputation | top-level `let` computed once before the first test | `Comb`, `Sieve` |
| W16 contexts | `with io(base: 1)` and user contexts | `io`, `checks`, `lint` |

Each entry in the spec also states how far `scripts/spec-check.sh`
already accepts it (checks, partial, or no), so an entry is a contract
with its implementation status beside it.

### What a library leaves to the contestant

The stdlib's area designs (`spec/lib/rationale/`) apply one test to every
candidate API: does it say something the editorial says, or does it hide
the part that is the problem? The answer draws a stable line:

- **DP**: the recurrence stays user code. `dp[k + 1, c] max= dp[k, c] +
  gain` is the editorial; the library offers only the optimizations
  (`knuth`, divide and conquer, convex hull) that take a cost operation.
  There is no `digit_dp` combinator, because the state differs every
  time (`spec/lib/rationale/dp.md` §4).
- **Graphs**: algorithms take a protocol (`Next`: a vertex's outgoing
  arcs), so `Graph`, a function `V -> Seq` and an implicit grid are all
  graphs. The transition of a state search, and a DFS whose per-vertex
  logic is the problem, stay user code (`graphs.md` §2, §4).
- **Trees**: one `Rooted` value owns a fresh domain of preorder
  positions in which every subtree and every heavy chain is one range,
  so there is no separate `Hld`, `EulerTour` or `Lca` type, and two
  trees' positions can't be mixed (`trees.md` §2). The tree DP's
  recurrence and the range structure's operation stay the user's
  (`trees.md` §5).

Each of these is built from powers in the table above: protocols with
associated types (W10a), generators (W6), fresh domains (W4) and
operation values (W10). A contestant who disagrees with a stdlib API can
write their own with the same tools.

## Key decisions

**The prelude is the chapters' vocabulary, by rule (decision 150).**
Lost: an 80-name budget with per-name review (decision 94). The budget
grew about one name per three requests until it was fiction, and
membership could only be learned by trying. A rule makes it guessable
from the docs. What costs a contestant is a collision, not a count, so
the safety comes from the two exclusions instead. The exported-signature
types rule closes the remaining gap: a lambda that annotates a value it
got from a prelude function must not need a `use`. Reverse: restore a
budget and per-name review in §11.3.

**Libraries get no prelude; inline modules do (decision 241).** A library
that relied on the importer's prelude would break when the prelude
changed. An inline `mod` in a program is part of that program, so making
it `use std.segtree` for a name the surrounding file already sees was
pure friction.

**Members are module-private unless `pub` (decision 358).** One
visibility rule for top-level names, fields and methods. Lost: always
public fields, which let a program reach into a library's invariant
(a DSU's parent array), and per-member keywords beyond `pub`. Inside one
module everything is visible, so a single-file solution is unaffected.

**One stdlib, written in the language (decision 28).** Every mechanism
the stdlib needed (reflection as a `for f in fields(x)` loop, variadic
packs, enum order) was added to the core for everyone, never
special-cased for `std/`. Reflection is a loop and not a macro or a
`comptime` block because it is the one construct a contestant already
reads as "for each", needs no new keyword, and keeps generated code
visible to the trace and the formatter. Lost
with it: Scala-style by-name parameters (`d: => T`) and an implicit
conversion from `T` to `() -> T`, because a call site must say when its
argument runs and how often. The stdlib ships with the compiler the same
way: `std/` is embedded as source, minus its `@test` fns (decision 72),
so `use std.x` resolves in a lone `cpl` binary and in the browser.

**Algebra is operation values, not traits per monoid (decision 3).**
`SegTree(+)`, `SparseTable(min)` and `Monoid(id, op)` pass the operation.
Lost: `impl Monoid for MaxInt` wrapper types, the design of most typed
libraries. Wrappers make the same tree over the same `Int`s with two
operations into two types and two impls, and contestants switch the
operation during revision more than any other part of a structure.

**Graphs are a protocol with associated types (decision 146).** Lost: one
concrete `Graph` type every algorithm takes, which forces implicit graphs
(grids, state spaces) to be materialized, and a `w:` argument on every
algorithm. Instead `dijkstra` is bounded by `WeightedNext<V, W>`, and
associated types, trait aliases and row bounds became powers so that a
user graph works the same.

**Patterns are closed (decision 132).** A view is a method returning an
enum, not custom destructuring. Custom patterns would make `match` run
user code with no visible call, which breaks "nothing secret under the
hood" (`handbook/principles.md`).

**No overloading (decision 127).** A function is its name plus its
labels, so options are named arguments (`by:`, `init:`), never a `_by`
twin, and a call resolves by reading it. Recursive `let` lambdas cover
the case overloading was most often wanted for.

**A fold costs what the receiver's type says (decision 326).** A generic
`.sum()` on a user `Seq` walks it; on a structure with an \(O(1)\) total
it is \(O(1)\). Costs are visible in each signature's `/// cost:` line
(W12), so a library type's cost is as discoverable as std's.

## Worked examples

Two modules, one using the other and the stdlib:

```cp
// output: "2 6"
mod a:
    pub fn f() -> Int = 1
mod b:
    use std.math.gcd
    use a.f
    pub fn g() -> Int = f() + 1
    pub fn h() -> Int = gcd(12, 18)
print(b.g(), b.h())
```

A user generator is a sequence too, with no `impl` at all:

```cp
// output: "4 2 10"
fn countdown(from: Int) -> yields Int:
    var i = from
    while i > 0:
        yield i
        i -= 1
print(countdown(4).max()!, countdown(4).count(x => x > 2), countdown(4).sum())
```

# Reference

## Where the precise rules are

| Topic | Rules |
|---|---|
| Program and library files, `mod`, member privacy | `spec/core/11-modules-prelude.md` §11.1, R11.1.1 to R11.1.4 |
| `use`, module graph, shadowing, inlining | §11.2, R11.2.1 to R11.2.4 |
| The prelude, exclusions, methods | §11.3, R11.3.1 to R11.3.5 |
| Stamps, pins, codemods | §11.4, R11.4.1 to R11.4.3 |
| What the submission contains | §11.5 |
| The powers | `spec/core/12-powers.md` W1 to W16 |
| Sequence capabilities and lending | W4, R12.4.1 to R12.4.13 |
| Associated types, aliases, field traits, row bounds | W10a, R12.11.1 to R12.11.4 |
| Stdlib area designs | `spec/lib/rationale/README.md`, `dp.md`, `graphs.md`, `trees.md` |
| Conformance cases | `spec/conformance/core-11/`, `spec/conformance/core-12/` |

## Changes

- 2026-11-23: first version.

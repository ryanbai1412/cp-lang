# The language in one read

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/00-overview.md` |
| Describes | `spec/core/`, `spec/lib/`, `std/prelude.cp` |

CP Lang is a language for competitive programming that compiles to one
C++17 file. This page is the whole design in one sitting: what it is for,
the handful of ideas a contestant holds in their head, and how they fit
together. Each idea has its own design doc, linked where it comes up; handbook leads design; spec/ is its downstream exhaustive form, so a
difference between this page and `spec/` is a bug to fix in one of them. Terms are defined in `spec/GLOSSARY.md`.

## The problem

A contest solution is written fast, changed often and judged once. Here is
the C++ a strong contestant writes for "shortest path from vertex 1 to
vertex n, or -1":

```cpp
int n, m; cin >> n >> m;
vector<vector<pair<int, long long>>> g(n + 1);
for (int i = 0; i < m; i++) { int u, v; long long w; cin >> u >> v >> w; g[u].push_back({v, w}); g[v].push_back({u, w}); }
vector<long long> d(n + 1, LLONG_MAX);
priority_queue<pair<long long, int>, vector<pair<long long, int>>, greater<>> pq;
d[1] = 0; pq.push({0, 1});
while (!pq.empty()) { auto [du, u] = pq.top(); pq.pop(); if (du != d[u]) continue;
  for (auto [v, w] : g[u]) if (du + w < d[v]) { d[v] = du + w; pq.push({d[v], v}); } }
cout << (d[n] == LLONG_MAX ? -1 : d[n]) << "\n";
```

It works, and it hides five facts the contestant knew and the compiler
never learned: vertices are 1..n and edges are a different thing; weights
are at most 10^9, so a path fits in 64 bits; `LLONG_MAX` means
"unreachable"; `int` was a guess at a width; the arrays are sized `n + 1` so
the 1-based input fits, and slot 0 is a dead value nobody checks. Each hidden fact is a bug class: an edge index
used as a vertex, an overflow, a sentinel added to, a wrong width. The
same program in CP Lang:

```cp
// input: "3 2\n1 2 5\n2 3 7\n"
// output: "12"
domain V = 1..=read<1..=2e5>()
let m = read<0..=2e5>()
let g = Graph.undirected(read<[m](u: V, v: V, w: 1..=1e9)>())
let p = dijkstra(g, from: [(V.first!, 0)])
print(p[V.last!] ?? -1)
```

Every fact is stated once, where the statement states it. The vertex
count bounds positions, so `V` is stored in 4 bytes; 1-based input stays
1-based; the reads validate the bounds in checked builds; the checker
proves a path sum of `2e5 * 1e9` fits in `Int`; and an unreachable
target is `None`, printed through `?? -1`, never `INF`.

## The mission and the five goals

The mission (`AGENTS.md`, `handbook/product/vision.md`) is to make
turning an algorithmic idea into a correct, efficient, accepted solution
as fast and natural as possible, for experienced contestants (about
Codeforces 1500+) who would replace a C++ setup tuned over years. Judge
the whole attempt, not the finished program: the DP that needs another
dimension, the binary search that must return an end position, the first
idea that was wrong.

Five goals shape the language. Each one rules out designs that look good
in a finished program and cost minutes in a contest.

- **Quick to write.** Say each fact once. The library names the textbook
  algorithm, and there is no ceremony: no `main`, includes or fast-I/O
  lines. You see it in `read<T>()`, the prelude and `@!multitest`.
- **Quick to revise.** Adding a dimension, a test loop or a second answer
  is a local edit, thanks to product domains `[R, C]T`, slices that keep
  their keys, and named options.
- **Common mistakes hard to write.** The bug classes C++ contestants hit
  most become type errors or traps, through domains, range types, `?T`
  and owned or lent values.
- **Failures easy to explain.** A check names the value, the bound and the
  source line, in the interpreter and the checked C++ alike.
- **The speed and control of hand-written C++.** Every abstraction lowers
  to the loop you would have written, and costs are visible: generators,
  monomorphized operation values, release builds with no checks.

## The model

Six ideas carry the whole language. Each is short to state, and each
removes a class of bug or ceremony without costing judge time.

### 1. Domains and keys: index spaces are types

A **domain** is a finite, ordered index type: `domain V = 1..=n`. Its
members are **positions**; arrays over it are `[V]T`, indexed only by a
`V`. Between positions sit **cuts** (`V.cuts`, n + 1 of them), and a
range `l..r` of positions has cuts for ends. Slices keep their source's
keys, so an index found inside a slice means the same element outside it.

```cp
// output: "40 40 3"
let a = [10, 20, 30, 40, 50]
let mid = a[2..5]
print(mid[3], a[3], mid.len)
```

A vertex can't index an edge array, a cut can't index a position array,
and the two Euler tours of two trees are different domains. Indexing a
`[V]T` with a `V` needs no check at all, because the type is the proof.
The design and its alternatives: [indices-and-keys.md](indices-and-keys.md).

### 2. Values and numbers: compute in 64 bits, store in the fewest bytes

Locals and arithmetic are `Int` (64-bit), so nobody tunes widths in a
register. Storage width comes from a type written for another reason: a
domain, a range type from the statement (`1..=1e9`), a `Mod<P>`. Every
type has spare values it never holds, and `?T` keeps `None` in one, so
`[V]?Int` is as wide as `[V]Int`. Plain values copy; heap values (`List`,
`Str`, arrays, maps) move and copy only with `.clone()`. Design:
[values-and-numbers.md](values-and-numbers.md).

### 3. Ownership: owned or lent, and `mut` at both ends

Every value is **owned** or **lent**. Parameters are lent for the call;
`mut` marks a mutated parameter at the declaration and at the call; a lent
value lives until its last use and can't outlive or race its source. There
are no lifetimes in signatures, only `-> lends T` on a function that
returns a view of a parameter.

```cp
// output: "2 3 4\n1 2 3"
fn bump(mut xs: List<Int>):
    for i in 0..xs.len: xs[i] += 1
var a = [1, 2, 3]
let before = a.clone()
bump(mut a)
print(a)
print(before)
```

Five rules replace a borrow checker's vocabulary. Design:
[ownership.md](ownership.md).

### 4. Sequences and views: generators push, views borrow

A **generator** (`fn f() -> yields T`) pushes elements into a loop body
and allocates nothing; a `for` over it compiles to the loop you would
have written, so `break`, `continue` and `return` act on that loop. A
**view** (a slice, `.rev`, `.map(f)`, `.filter(p)`) is a sequence built
from a source without copying it. Library operations ask for the weakest
of six capabilities (`Seq`, `Keyed`, `Sized`, `Bidir`, `Indexed`,
`IndexedMut`), so an \(O(n)\) walk never hides behind `s[k]`.

```cp
// output: "8 1"
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
print(evens(10).max()!, [3, 9, 2].argmax()!)
```

Design: [control-and-functions.md](control-and-functions.md) for
functions, lambdas and generators, [sequences.md](sequences.md) for views
and the capabilities.

### 5. Traits and operation values: algebra is a value

A trait names a capability (`Ord`, `Next<V>`, `Monoid<T>`) and may state
**laws** that stress testing samples. Operations are values with laws:
`+`, `max`, `gcd` and user monoids are passed to segment trees, sparse
tables, folds and scans, which pick a documented specialization from the
laws. There is no overloading: one name, one function.

```cp
// output: "10 4"
let a = [3, 1, 4, 2]
let sums = SegTree(+, a)
let maxes = SparseTable(max, a)
print(sums.fold(0..4), maxes.fold(1..3))
```

Design: [traits-and-generics.md](traits-and-generics.md).

### 6. Contexts and checks: settings are lexical, checks have two builds

A **context** is a library-declared record of compile-time settings, set
for the rest of a block by `with`: `with io(base: 1)`,
`with checks(overflow: on)`, `with lint(name: allow)`. One mechanism
replaces a flag per feature. Every checked operation answers three
questions: what it means, what the compiler proves, and what each build
does. Checked builds (`cpl run`, tests, stress) trap on every unproven
fact; release builds for the judge run no checks by default. Design,
together with input and output: [checks-and-io.md](checks-and-io.md).

### And: the library has no powers you lack

Everything above is used by `std/` through public mechanisms, and a user
library can use every one of them: a user type that implements `Seq` gets
every fold, a user struct with `set` gets `x[k] += v`, a user context
works with `with`. The prelude is one ordinary file of `use` lines. Design:
[modules-and-powers.md](modules-and-powers.md); the library's own design:
[std.md](std.md).

## How the ideas compose

The ideas are useful because they multiply. A typical solution uses all
six in a few lines, and each one makes the others cheaper:

- **Domains + numbers.** A domain declared from a bounded read gives both
  the index type and its storage width: `[V]Int` indexed by `V` needs no
  check, and `[V]?V` for parents is 4 bytes a slot.
- **Domains + sequences.** Views keep keys, so `a[l..=r].argmax()`
  answers with a key of `a`, and prefix scans are indexed by cuts:
  `pre[c]` folds everything left of `c`.
- **Ownership + sequences.** A view is just a lent value; the same five
  rules decide how long `a.filter(p)` may live, with no `View` trait.
- **Traits + library powers.** `dijkstra` takes any `Next<V>` whose arcs
  carry a weight: a stored `Graph`, a grid, a closure over states.
- **Contexts + checks + I/O.** `with io(base: 1)` converts counted
  domains at every read and print, so the arithmetic `- 1` never appears
  in the program, and folds away at compile time.

```cp
// input: "2\n3\n5 1 4\n2\n7 7\n"
// output: "5 1\n7 1"
@!multitest
domain I = 1..=read<1..=2e5>()
let a = read<[I]1..=1e9>()
let best = a.argmax()!
print(a[best], best)
```

Here the domain sizes the array and types the answer, the range type
validates the input, the fold answers with a key (printed as its integer,
1-based as the statement wants), and `@!multitest` makes the same
program handle many cases.

## Key decisions

The decisions that most define the language's character, with the
alternatives that lost. Numbers refer to entries in `decisions.md`.

| Decision | Chosen | Lost, and why |
|---|---|---|
| Index spaces (2, 137, 143) | Fresh domain types; positions vs cuts; slices keep keys | Bare `Int` indices: the most common C++ contest bug class stays. Slices re-keyed from 0: an index found in a slice would name a different element outside it (`spec/core/04a-identity-vs-offset.md` walks every option) |
| Widths (149) | Compute in `Int`, store in the width the types imply | Manual `u32` fields: the width is a second copy of a fact already written as a range |
| Absence (149, chapter 3) | `?T` in a niche, never a sentinel | `-1`/`INF` sentinels: they get added to, compared and printed by accident |
| Ownership (128, 141, 144) | Owned or lent, five rules, `lends` only on signatures | Implicit deep copies (hidden \(O(n)\) work); Rust-style lifetimes and borrow modes in types (a vocabulary that costs minutes in a contest) |
| Iteration (126, 142) | Push-only generators and six capabilities | Pull iterators (`next()`): `break` and `return` through a library loop need machinery; one iterator trait hides \(O(n)\) reads behind `s[k]` |
| Overloading (127) | None: a function is its name plus its labels | Overloads make errors and completion ambiguous |
| Completion (513) | Offers only what the checker accepts at the cursor | A hand-kept table: it drifts from the checker and offers code that does not compile |
| Algebra (3) | Operation values: `SegTree(+)`, `SparseTable(min)` | A zoo of wrapper types (`Sum`, `Max`, `Semiring`) to learn and import |
| Checks (100, 148) | Trap in checked builds, no checks in release by default | Checks on in release: they cost judge time and turn a wrong answer into another wrong answer |
| Library (`AGENTS.md`, chapter 12) | No privileged stdlib | Compiler special cases: user code could never feel as built in, and every special case is a rule to learn |
| Prelude (150) | Exactly the `lib/` chapters' main tables | A hand-picked budget: membership becomes a negotiation nobody can predict |

Two rules run through all of them. **Costs are visible:** a member read
without parentheses is \(O(1)\), anything that allocates or walks looks
like a call, and `cpl explain` shows every implicit choice (widths,
layouts, specializations, copies). **Two builds, one meaning:** a valid
program behaves the same in the interpreter, the checked C++ and the
release C++.

## Settled decisions

These change only with strong evidence from the corpus. Each links to the
doc that gives its reasons.

- **Surface syntax.** Indentation blocks, `and`/`or`/`not`, `elif` (never
  `else if`), `=>` match arms. Ranges are Rust-style: `0..n` is half-open,
  `0..=n` inclusive. See [control-and-functions.md](control-and-functions.md).
- **Every integer has a range type.** Ranges come from fixed rules
  (annotations, reads, `D.count`, interval arithmetic, hulls), with no flow
  narrowing. An index the compiler can't prove stays checked in the
  submission and traps. See [values-and-numbers.md](values-and-numbers.md).
- **Arithmetic.** Overflow traps in checked builds. `/` and `%` are
  Euclidean, so `-7 / 2` is `-4` and `-7 % 2` is `1`, and `%` is never
  negative. `Int` excludes the lowest 64-bit value, so `?Int` costs no extra
  space. `F64` `==` is exact, and `close(a, b, eps:)` is the tolerant
  compare. See [values-and-numbers.md](values-and-numbers.md).
- **Optionals.** `?T` absorbs arithmetic (`None + 5` is `None`), and
  `let ... else` and postfix `?` take the value out. See
  [values-and-numbers.md](values-and-numbers.md).
- **Domains from runtime data.** `domain V = 0..n` with a read `n` gets one
  hidden slot per declaration, saved and restored on recursion. Its values
  leave their block only through an existential return type. See
  [indices-and-keys.md](indices-and-keys.md).
- **Stack and storage.** `main` runs with the judge profile's stack, so
  there is no depth inference and no depth annotation. Runtime-sized and
  large arrays live off the stack, and returns move. See
  [ownership.md](ownership.md).
- **`Bool` is not printed bare.** Write `print(yes(b))`, so the judge's
  expected word is always explicit. See [checks-and-io.md](checks-and-io.md).
- **Pointer structures use typed handles** into a library-owned store,
  with no pool syntax. See [modules-and-powers.md](modules-and-powers.md).
- **Generics are checked at their definition,** with errors that libraries
  can extend. See [traits-and-generics.md](traits-and-generics.md).
- **Brute force, generators and checkers are ordinary functions** in the
  same file, not special file sections. `@!multitest` on its own line
  divides the shared setup from the per-test body. See
  [checks-and-io.md](checks-and-io.md).

# Reference

## Where the precise rules are

| Topic | Spec | Design doc |
|---|---|---|
| The model in one page | `spec/core/00-overview.md` | this page |
| Lexical grammar | `spec/core/01-lexical-grammar.md` | |
| Values, types, numbers, layout | `spec/core/02-values-types.md`, `03-numbers-layout.md` | [values-and-numbers.md](values-and-numbers.md) |
| Domains, positions, cuts, keys | `spec/core/04-indices.md`, `04a-identity-vs-offset.md`, `04b-keys.md` | [indices-and-keys.md](indices-and-keys.md) |
| Ownership | `spec/core/05-ownership.md` | [ownership.md](ownership.md) |
| Control flow and functions | `spec/core/06-control-functions.md` | [control-and-functions.md](control-and-functions.md) |
| Sequences | `spec/core/07-sequences.md`, `spec/lib/sequence-patterns.md` | [sequences.md](sequences.md) |
| Traits and generics | `spec/core/08-traits-generics.md` | [traits-and-generics.md](traits-and-generics.md) |
| Contexts, checks, I/O | `spec/core/09-contexts-checks.md`, `10-io.md` | [checks-and-io.md](checks-and-io.md) |
| Modules, prelude, powers | `spec/core/11-modules-prelude.md`, `12-powers.md` | [modules-and-powers.md](modules-and-powers.md) |
| Lowering to C++ | `spec/core/13-lowering.md` | this page; decisions 31 (stack, layout, niches) and 52 (static closures) |
| The standard library | `spec/lib/` | [std.md](std.md) |

Every spec chapter, core and `lib/`, opens with a short overview ("The
model in one paragraph"); the design doc for the area starts from it and
says why. Debates older than the handbook are in
`research/design-history/` (`decisions-design.md`, `style-decisions.md`).

## How math is written (decision 487)

Costs and bounds are math, and the spec, doc comments and lint records
write them as LaTeX inside `\( ... \)`: `\(O(n \log n)\)`, with sizes as
`\lvert D \rvert` rather than `|D|` and multi-letter names as
`\mathit{len}`. The site typesets it, and hover and CLI output show a
plain fallback such as `O(n log n)`. Code comments stay as written.

```cp
/// The sum of `a`.
/// cost: \(O(\mathit{len})\)
fn total(a: List<Int>) -> Int = a.sum()
print(total([1, 2, 3]))  // a comment stays plain: O(n)
```

Plain text such as `O(n log n)` or `|D|` lost because it cannot be
typeset, collides with table pipes and is spelled differently by each
writer; Unicode math such as `O(n·α(n))` is hard to type and to check.
LaTeX is the one form that renders, checks and still reads in source.

## Changes

- 2026-11-23: first version.
- 2026-11-23: settled decisions folded in from the operating notes, corrected to the current spec (Euclidean division, exact `F64` `==`, `@!multitest`).

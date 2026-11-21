# Spec chapter 1: index spaces, domains and scoped types

Status: **draft v2**, distilled from the round-9 exemplars (`EXEMPLARS.md`, `ACCEPT.md`). Everything here is
[proposed] until Ryan locks it. Tags: `[you]` = Ryan decided; `[rN k]` = review round N item k;
`[unexercised]` = carried over from SPEC v1 but no exemplar uses it yet, so the ports must justify it.

## 1.1 Why

Most contest WA/RE bugs are an integer used in the wrong place: an element index where a prefix index was
meant, a vertex used as an edge, a 1-based value used 0-based, `i + 1` off the end. The language makes each
of these a type error by giving every index space its own type, and puts the remaining arithmetic behind
named operations whose boundary behaviour is visible.

## 1.2 The three kinds

| kind | declared | identity | `io(base:)` converts | compares with `Int` | arithmetic |
|---|---|---|---|---|---|
| counted | `domain V = n` | nominal, fresh | yes | no (`v.ord`) | none |
| labeled | `domain G = labels(1..=3)`, `labels("a", "b")` | nominal, fresh | no; keeps `.label` | no (`g.label`) | none |
| integer range | a type like `1..=n`, `0..n` | structural (`Int`) | no | yes | ordinary `Int` |

- **Counted** is the default for anything the statement numbers 1..n: vertices, positions, queries.
- **Labeled** is the cold path for values whose written form matters (`labels(1..=3)` for groups A/B/C,
  `labels("R", "G", "B")`). Its labels never convert, so mixed-base input is one local edit [you].
- **Integer range** is for values you compute with: lengths, sums, digit positions. `[1..=n]T` is an array
  indexed by an integer range (`p.inverse()` returns one).

`domain X = …` always makes a **fresh** type, even for the same size. `type X = …` makes an alias.

```cp
domain V = read(1..=2e5)
domain E = read(1..=2e5)
var deg: [V]Int = 0
let ends: [E](V, V) = read()
for (u, v) in ends:
    deg[u] += 1
    deg[v] += 1
deg[e]          // error: `deg` is indexed by `V`, got `E`
```

## 1.3 Declaring a domain

- **From a size you read** [r8 1]: `domain V = read(1..=1e5)`. In a tuple, the binding keyword marks the
  domain: `let (domain A, k): (1..=1e6, 1..=A.count) = read()`. Tuple components are scoped left to right,
  so later types may mention earlier names [r9 20].
- **From a size you have**: `domain V = n`, `domain V = gen(1..=8)`.
- **From a function that makes one** (fresh, existential): `let (domain Piece, pieces) = t.components(
  without: c)`, `let (domain X, xs) = compress(vals)`. The keyword is required; a capitalized name alone is
  rejected (too implicit) [r2].
- `V.count` is the size, typed by the declared range (`1..=1e5`), so `[V.count - 1]` and `let n = V.count`
  keep their bounds [r10 30]. An existential domain inherits its source's maximum [r10 28]. When the size is used often, `let n = V.count` is the idiom [r9 19].
- **Sizes never convert** under `io(base:)`; only positions do [r9 16].

## 1.4 Scope and lifetime

- Domains are **block-scoped** and declared before use [you]. A value whose type mentions `V` can't leave
  `V`'s block: not by return, not into a `static`, not across iterations of the enclosing `@multitest`
  body. This is what makes "an index from test 3 used in test 4" a compile error [you].
- **Generic code receives the size as a hidden argument.** Wherever `D` is in scope, so is `D.count`, and
  the caller passes it to any generic instantiated at a runtime-sized `D`. So `for x in D`, `[D]Bool` and
  `Index.count()` work inside generic code. Debug checks that each `[D]T` argument has that length.
  `cp explain` shows the hidden argument [r4 18, r5 12, r10 14, 15]. A domain parameter that no value
  parameter mentions is passed explicitly, `f<Col, Row>(r)` [p1 S3].
- **Tool roots** (`@interactor`, `@gen`, `@check`) live at program level and can't capture a runtime-sized
  domain [r6 4]. `@alt` can, because it is substituted at its call site.

## 1.5 Scoped runtime types

The same rule covers any type parameterized by a runtime value [r6 6–10]:

```cp
let p: Prime = read()
type Z = Mint(mod: p)        // fresh, block-scoped, like `domain V = n`
var fact: [0..=n]Z = 1
```

- Code that takes a `Z` receives `p` as a hidden argument per activation, the way a domain's size travels,
  so recursion and reentry are safe. When no reentry is possible the compiler may use a static instead;
  `cp explain` says which.
- `Z` values can't leave the block. **One rule for both:** a generic with statics or `@memo` can't be
  instantiated with a scoped type or a runtime-sized domain, which is what makes a table built under one
  modulus (or size) and read under another a type error [r10 16].
- With a constant (`Mint(mod: 998244353)`) the type is specialized; a runtime modulus uses Barrett
  reduction. Going from the constant to `p` is one local edit.

## 1.6 Positions

Every position `v: D` of a counted or labeled domain is stored as its 0-based ordinal, in the narrowest
unsigned integer that holds `D`'s declared maximum (`u32` for `1..=2e5`). Operations:

| operation | type | meaning |
|---|---|---|
| `v.ord` | `0..D.count` | 0-based ordinal; the only way to an `Int` |
| `v.label` | label type | labeled domains only |
| `D.at(i)` | `D` | from an ordinal; checked |
| `D.get(i)` | `?D` | from an ordinal; fallible |
| `D(x)`, `D.try(x)` | `D`, `?D` | labeled domains: from a label (`Letter('c')`, `G(2)`). One conversion family everywhere: `T(x)` is checked, `T.try(x)` is fallible (`I32.try(x)`). `.at`/`.get` are only ever ordinals [p1 S1]. `V(x)` on a counted domain is an error naming `.at`; `Int(f)` stays an error, and an infallible conversion has no `.try` [r13 P10]. A lint flags `.at(literal)` on integer-labeled domains, whose diagnostic shows both readings [r13 P11] |
| `D.mirror(v)` | `D` | the position at the same distance from the other end [p1 F5] |
| `v.next`, `v.prev` | `?D` | neighbour or `none` at the end (differs from `fwd(1)` at the end) |
| `v.fwd(k)`, `v.back(k)` | `D` | up to k steps, **clamped** at the ends; total |

**No `v + k` on a linear domain.** `+ k` hides whether the end should give `none`, trap, clamp or wrap. The
error lists `next`/`fwd`/`back`/`D.at(v.ord + k)` [carried from v1]. A cyclic domain (`domain K = ring(n)`) has a total
`i + k` / `i - k` (wrapping), `i.dist_to(j): 0..n` (forward distance) and `K.walk(from: i)` visiting all n
positions. `D.as_ring()` is a distinct type `Ring<D>` (`D.ring(v)`, `r.pos: D`), so a linear `D` never gets `+`.
Rings have no `Cut` [p2 Q2, proposed default; r14 T8].

## 1.7 Cuts

Prefix sums, prefix DP, insertion points and interval splits live in the gaps between elements. For a
domain `A` of size n, `A.Cut` is a derived domain of n + 1 cuts.

| operation | type | meaning |
|---|---|---|
| `x.before`, `x.after` | `A.Cut` | the gap left/right of element `x` |
| `c.left`, `c.right` | `?A` | the element left/right of cut `c`; `none` at the ends |
| `Cut.first`, `Cut.last` | `Cut` | before everything / after everything; always total |
| `Cut.rest` | `Range<Cut>` | every cut except `Cut.first` |
| `c.back(k)`, `c.fwd(k)` | `Cut` | clamped |
| `b.window(k)` | `Range<Cut>` | the ≤ k cuts strictly before `b`; empty at `Cut.first` |
| `c.window(k, incl: true)` | nonempty `Range<Cut>` | the ≤ k cuts ending at `c` inclusive |
| `a[r]`, `r: Range<Cut>` | slice | elements from `r.first` to `r`'s **end cut**: the written bound, whether `..` or `..=`. So `a[c .. c']` and `a[c ..= c']` are the same slice [r12 R1] |
| `r.before`, `r.after` | `Cut` | the cuts around a range of elements |
| `c.ord` | `0..=n` | elements before `c` (one name for the number, as for positions) [r10 18] |

- Cuts are counts, not positions, so they **never convert** under `io(base:)` [22].
- `A` and `A.Cut` are distinct, so `pre[x]` for a prefix array `pre: [Cut]Int` is a compile error; you write
  `pre[x.after]` or `pre[x.before]` and the off-by-one is a choice you can see.
- **Every array and slice has cuts** [p1 F1, r13 P20]: an anonymous `[n]T` or a slice like `g.adj(v)` has
  `xs.Cut`, a fresh nominal cut domain (`xs.Cut.at(i)` converts an `Int`), so `pre[i]` vs `pre[i + 1]` stays
  checked while elements stay `Int`-indexed. Slices of a named domain keep `A.Cut`.
- **Open ranges** [p1 F2]: `a[c ..]` (from cut `c` to the end), beside the prefix forms `a[..c]` / `a[..=c]`.
- **Reversed views** [p1 F5, r13 P21]: `xs.rev()` keeps `D` and reverses iteration order only; suffix work
  uses `suffix_*` scans, and `D.mirror(v)` names the opposite position, so the far end needs no `n - 1 - i`.
- `A.chunks(k)` yields `Range<A>` blocks of k (the last may be short) [Tourism].

## 1.8 Ranges

- `Range<D>` is stored half-open, `(first, end)`. It is written `a .. b` (half-open, as in Rust) or `a ..= b` [you, p3: was `..<`]. A reversed range is
  empty. `r.first`/`r.last` are `?D` unless the range is statically nonempty (§1.10).
- A `Range` reads and prints as `first last`, inclusive, converting each end under `io(base:)` (rule 4).
- `r & s` intersects; `..=c` and `..c` are prefix ranges, so `b.window(k) & (..=e)` clips a window [r10 20]. Prefix and suffix forms (`..=c`, `..c`, `c ..`) are unary at range
  level and take parentheses after a binary operator [r13 P2]. There is no bare `.. c` [r13 P18].
- A domain converts to its full range, a position to a one-element range, and a slice to its range,
  wherever a `Range<D>` is expected (`t.fold(I)`, `t.apply(x, tag)`, `h[w]` for a window `w`) [r10 3, r11 N33].
- Slicing by cuts is defined once, in the table above, so `a[prev]` with `prev = c ..= c'` needs no
  `prev.first .. prev.last` [r11 N35, r12 R1].
- **Sizes:** `.count` is only an index space's size (`V.count`); every sequence and container uses `.len`
  [r11 N9].

## 1.9 Arrays over index spaces

- `[D]T` is a heap array indexed by `D`. Indexing with a `D` needs no bounds check, because no `D` value can be
  out of range. Indexing with anything else is a type error.
- **An anonymous length `[n]T` is indexed by the integer range `0..n`** [r8 2], so `xs[i]` works with an
  `Int` and is bounds-checked (trap). Only named domains are nominal. `[n - 1]Edge<V>` is anonymous; its
  elements carry `V`s.
- `len:` always means "a length decided by the input, within this bound". An array isn't self-delimiting, so
  `[len: 1..=n]T` reads a length and then that many items; `Str(len: 1..=1e4)` needs no prefix [r12 R10]. The array is nonempty and anonymous [r8 5]. A lint
  flags `[1..=n]T` in read position, because it is one token from `[len: 1..=n]T` [r9 15].
- **Multi-dimensional arrays** [p1 S6]: `[A][B]T` is one dense row-major buffer; `g[i]` is a row slice and
  `g[i] = g[j]` copies a row element-wise. Rows are fixed-length slices, so they are never invalidated [r13 P25].
  `cp explain` reports total bytes.
- **Grid domains** [p1 F3]: `domain Cell = grid(r, c)` is a counted domain over (row, col) with `cell.row`,
  `cell.col`, `cell.neighbours` (4-way; `diag: true` for 8). `domain Node = Cell + labels("S", "T")` adds
  named extra positions, so flow sources need no `V - 2`. `Cell` is fresh and isomorphic to
  `(Cell.Row, Cell.Col)`, associated domains like `A.Cut`, with `cell.row: Cell.Row` and `cell.pair` [r14 T5].
  In a sum domain, cells come first, then labels; `Node.Cut` is a compile error [r14 T23]. `Node(c)` injects a cell and `n.cell: ?Cell` projects back. Under `io(base:)`
  cells convert and labels don't. `read<[Cell]Byte>()` reads r rows of c characters with no separators [r13 P22].
- `D.map(f)` builds a `[D]T`; a domain counts as the array of its positions [r4 5].
- `[D]T` is (pointer, length); `List<T>` adds capacity. `.list()`, `return` and assignment from a temporary move
  the buffer; `.list()` on a named array copies, and `cp explain` reports it [r13 P5].

## 1.10 Nonempty by construction

`.first`, `.last`, `.max()`, `.min()` and folds return `?T` unless the sequence is **syntactically**
nonempty; no prover changes a type [r3]. Syntactically nonempty means one of:
- a domain or array whose size type has lower bound ≥ 1 (`domain V = read(1..=n)`, `[k]T` with `k: 1..`);
- `Bits<D>` as a domain (it always has the empty set) [r14 T24]; `!` on a non-optional is an "already total" lint;
- a value of type `NonEmpty<…>`: `c.window(k, incl: true)`, `Bits<D>.nonempty`, `s.subsets()`,
  `s.windows(m)` with `m: 1..=s.len`, `[len: 1..=n]T`, a `Range` read from input (reading traps unless
  l ≤ r), `Range(x)` for one position, and `max_spans` results [r8 3, r10 8]. `NonEmpty<T>` is one wrapper
  for ranges, arrays and sets, so it can be written in signatures and fields (`[I]NonEmpty<Range<I>>`)
  [r10 44]. It converts implicitly to the plain type; the way back is `r.nonempty() -> ?NonEmpty<…>`, usually
  in `if let s = r.nonempty():` [r11 N1]. `r.hull(s)` is the smallest range covering both, with two
  signatures: `(NonEmpty, Range) -> NonEmpty` (either order) and `(Range, Range) -> Range` [r11 N2, r12 R8, R9];
- `map`/`zip` of a nonempty sequence. `where` and `filter` drop nonemptiness [r9 12].

Anything else needs `!` (trap) or `??` (default) at the use, which is where the reader wants to see it.

## 1.11 The `Index` trait

Every index space implements one trait [r7 18]:

```cp
trait Index:
    fn count() -> Int
    fn ord(self) -> 0..Self.count
    fn from_ord(i: 0..Self.count) -> Self
```

Counted, labeled and integer-range spaces, `A.Cut`, and `Bits<D>` implement it, so `[I]T`, `for x in I`,
`SegTree<I, …>` and `Bits<I>` work for every `I`. Only counted domains participate in `io(base:)`.
`Bits<0..n>` is legal [r9 13]. User types may implement `Index`; `from_ord` must be total on its domain, and
`cp test` checks `from_ord(ord(x)) == x` on random values.

## 1.12 `Bits<D>`: subsets of a small domain

`Bits<D>` is a set of `D`s in one machine word; `D`'s **static** maximum must be ≤ 64, a compile error
otherwise [r10 28]. It is an
index space (`[Bits<R>]Int` has 2^|R| entries) and a set:

```cp
var score: [Bits<R>]Int = 0
for s in Bits<R>.nonempty:
    let (x, rest) = s.pop_min()          // total: s is nonempty by type
    score[s] = score[rest] + rest.map(y => a[x][y]).sum()
```

Operations: `contains`, `s + x` / `s - x` for one element, `|` / `&` / `-` for sets, `min`/`max`
(total on nonempty), `len`, `pop_min`, `subsets()`, `D.mask(xs)`, `.full`, `.empty`, `.nonempty`; iterate the type
itself for every subset [r10 7]; raw word via `s.raw` /
`Bits<D>.from_raw(w)` for bit tricks. Iteration order is increasing raw value, so `s` comes after every
proper subset; a DP over subsets in `for s in Bits<D>` order is valid [ACCEPT grouping].

- `Bool` implements `Index` with count 2, so `[Bool]T` and dense `@memo` over a `Bool` work [r11 N4].

- **Subsets as domains** [p2 F4]: `xs.numbered() -> (domain K, [K]T)` [r14 T16] numbers a filtered subset; `.inverse()` on
  a `[K]D` gives `[D]?K`. `compress` and `g.components()` are instances. Nonempty sources give a nonempty `K`.
- **Product domains** [p2 Q1, proposed default]: in type position a tuple of index spaces is an index space.
  `[(A, B)]T` is the same type as `[A][B]T` (diagnostics print `[A][B]T`), so `t[(a, b)]` and `t[a][b]` both
  work. Iteration is `product(A, B)`, never a `(A, B)` value [r14 T12, T13]. Hidden sizes and `io(base:)` act
  per component. `Bits<(A, B)>` needs the static product ≤ 64. `grid` stays a fresh domain (§1.9) whose
  `Cell(r, c)` builds a cell.

## 1.13 Permutations and labeled destructuring

- `Perm<I, S>` is a bijection from index space `I` onto `S`; `I` defaults to `[n]`. Reading one checks
  bijectivity. `p.inverse(): [S]I` [r10 39].
- A labeled domain with a static count destructures: `let [small, mid, big] = Group.sorted(by: want)` [r10 40].
- `grid` and `Cell + labels(…)` are adopted (§1.9) [r13 P13]; `ring` is adopted (§1.6) [p2 Q2]; `heap_tree` stays in the ports backlog;
  each must remove friction in ≥ 5 ports to come back [r10 47].

## 1.14 Lowering

| construct | C++ |
|---|---|
| position `v: V` | `uint32_t` (narrowest unsigned that holds the max) |
| `?V` | same width, niche value `UINT32_MAX` |
| `[V]T` | `T*` + count in a heap buffer (never a VLA on the stack) |
| `A.Cut` | `uint32_t` holding the count |
| `Range<D>` | two ordinals, half-open |
| hidden domain size | an extra `uint32_t` parameter, only where generic code needs it |
| `Bits<D>` | `uint64_t` (or narrower when `D.count` is statically small) |
| `Mint(mod: const)` | value + compile-time modulus; runtime modulus: value + per-activation Barrett constants |

Checks remaining after typing: conversions from `Int` (`D.at`, reads), anonymous-array indexing, and range
contracts. All trap with the source line and values in your terms (one-based if `io(base: 1)`).

## 1.15 Diagnostics (required wording shape)

- Wrong domain: "`deg` is indexed by `V`, but `e` is an `E`. `V` was declared at line 3, `E` at line 4."
- Cut vs element: "`pre` is indexed by `A.Cut`; `x` is an element of `A`. Did you mean `pre[x.after]`
  (prefix through `x`) or `pre[x.before]` (prefix before `x`)?"
- Escaping domain: "`order: [V]V` can't be returned: `V` is local to the test body at line 2."
- `v + 1`: "positions have no `+`. Use `v.next` (`none` at the end), `v.fwd(1)` (clamped), or `V.at(v.ord + 1)`
  (checked)."

## 1.16 Tool coverage

Every construct here must work in: the debug interpreter (positions shown as labels, one-based on request),
the HTML tracer (arrays shown with their index space, cuts drawn between cells), `cp explain` (widths,
hidden arguments, remaining checks), stress generators (`gen<Graph<V>>`, `gen(1..=8)` for sizes), the
formatter, and the LSP (hover shows the index space of every subscript).

## 1.16b Round-3 port rules [p3]

- Positions of one domain are `Ord` (by ordinal). `Bool` is a labeled index space (`[Bool]T`, `(Wall, Bool)`).
- `v.fwd(k)` clamps at the last position, like `v.back(k)`. `.next`/`.prev` are the optional forms.
- One-sided position ranges: `p ..` (p to the last), `.. p` (before p) and `..= p` (the first to p), the same
  grammar as cuts [r16 R6]. On an `Int`-indexed sequence `xs[.. i]` is positions `0..i`.
  `D[c .. c']` (a domain sliced by a cut range) is a `Range<D>`.
- `v.written` is the ordinal as written in the current `io(base:)`, for answers that sum or XOR indices (`uses io`).
- `i.dist(j)` is the shorter distance (`|i.ord - j.ord|` on linear domains); `i.steps_to(j)` is the forward
  distance on rings.
- `ring(labels(N, E, S, W))` is a labeled ring with wrapping `+`/`-`/`walk`, for grid directions.
- **Storage:** `[D]T` whose `D.count` is static and whose size is ≤ 4 KB is stored inline, like a C array: on the
  stack, in the enclosing struct, or in a segment-tree node. Larger static and all runtime-sized arrays use the
  heap. A read array whose `len:` bound is a literal ≤ 8 is inline. On the stack, inline storage is used only
  in non-recursive functions or when inferred depth × size fits the stack budget; otherwise heap (6.5 "arrays
  never go on the stack" still holds for anything unbounded). `.list()` on an inline array copies, and the
  large-copy lint covers inline values. `cp explain` shows which.
- A product-domain position is stored as one ordinal in the narrowest unsigned type, with a niche for `none`.
  Unpacking divides by the inner count: a shift for static powers of two, a Barrett constant for runtime
  counts, reported by `cp explain` [r16 R21].
- User structs, impls and generic instantiations that mention a scoped type (a fresh domain, `Mint(mod: p)`)
  are scoped with it. A program-level declaration such as `enum Query<V>` is unscoped; only its instantiations
  at a scoped `V` are scoped.
- `Bool`'s labels are `0`/`1` for I/O, so it reads and prints like any labeled domain.

## 1.17 Open questions

1. `A.Cut` (a type, capitalized like one) vs `A.cuts`. Reviewer: keep `A.Cut`.
2. `D.at(i)` vs `D(i)`. Reviewer: keep `at`; `D(x)` is for meaning-preserving conversions like `I32(x)`.
3. Do product/sum/shape domains survive the ports? (Backlog.)
4. ~~Nonempty as a flag or a type?~~ A type, `NonEmpty<…>` (§1.10) [r10].

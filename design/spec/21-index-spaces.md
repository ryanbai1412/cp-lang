# 21. Index spaces

**Layer:** library over C6/C7. Everything here could ship in a user library; the core parts are in `12` and `13`.

Status: [proposed].

## 21.0 What's core and what's library

Core provides only fresh scoped types (chapter 12) and `[D]T` over the `Index` trait (chapter 13).
Everything in this chapter is library code over those two, so a user library could add a new kind of index
space with the same power. The core of each kind:

```cp
struct Counted(count: Int)                         // `domain V = n` ≡ `let (domain V, _) = Domain.of(n)`
impl Index for Domain.of(Counted):
    fn count() = Self.size                         // the hidden runtime parameter (chapter 12.3)
    fn ord(self) = self.raw                        // stored in the narrowest unsigned type
    fn from_ord(i) = Self(raw: i)

impl<A: Index> Index for A.Cut:                    // n + 1 gaps
    fn count() = A.count + 1
impl<A: Index> A:
    fn after(self) -> A.Cut = A.Cut.from_ord(self.ord + 1)
    fn before(self) -> A.Cut = A.Cut.from_ord(self.ord)
impl<A: Index> A.Cut:
    fn left(self) -> ?A = A.get(self.ord - 1)

fn grid(r: Int, c: Int) -> (domain Cell)           // a fresh domain over (Row, Col)
impl<D: Index> D: fn next(self) -> ?D = D.get(self.ord + 1)
```

`labels(a, b)` is sugar for a payload-free enum deriving `Index` and `Read`/`Print` by label. `ring(n)`,
`Bits<D>`, `Perm<I, S>`, `numbered`, `compress` and `components` are ordinary types and functions; the last
three return fresh domains existentially (chapter 12.2).

**Why it's worth showing the code.** "Is `v.next` special?" has a one-line answer: no, it's `D.get(v.ord + 1)`,
and the absence of `v + 1` is the absence of an `Add` impl, not a compiler rule.

## 21.1 Why

Most contest WA/RE bugs are an integer used in the wrong place: an element index where a prefix index was
meant, a vertex used as an edge, a 1-based value used 0-based, `i + 1` off the end. The language makes each
of these a type error by giving every index space its own type, and puts the remaining arithmetic behind
named operations whose boundary behaviour is visible.

## 21.2 The three kinds

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

## 21.3 Positions

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

## 21.4 Cuts

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

## 21.5 Ranges

- `Range<D>` is stored half-open, `(first, end)`. It is written `a .. b` (half-open, as in Rust) or `a ..= b` [you, p3: was `..<`]. A reversed range is
  empty. `r.first`/`r.last` are `?D` unless the range is statically nonempty (21.6).
- A `Range` reads and prints as `first last`, inclusive, converting each end under `io(base:)` (chapter 25).
- `r & s` intersects; `..=c` and `..c` are prefix ranges, so `b.window(k) & (..=e)` clips a window [r10 20]. Prefix and suffix forms (`..=c`, `..c`, `c ..`) are unary at range
  level and take parentheses after a binary operator [r13 P2].
- A domain converts to its full range, a position to a one-element range, and a slice to its range,
  wherever a `Range<D>` is expected (`t.fold(I)`, `t.apply(x, tag)`, `h[w]` for a window `w`) [r10 3, r11 N33].
- Slicing by cuts is defined once, in the table above, so `a[prev]` with `prev = c ..= c'` needs no
  `prev.first .. prev.last` [r11 N35, r12 R1].
- **Sizes:** `.count` is only an index space's size (`V.count`); every sequence and container uses `.len`
  [r11 N9].

## 21.6 Nonempty by construction

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

## 21.7 `Bits<D>`: subsets of a small domain

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
- **Product domains:** see chapter 13.3. `grid` stays a fresh domain (below) whose `Cell(r, c)` builds a cell.

## 21.8 Permutations and labeled destructuring

- `Perm<I, S>` is a bijection from index space `I` onto `S`; `I` defaults to `[n]`. Reading one checks
  bijectivity. `p.inverse(): [S]I` [r10 39].
- A labeled domain with a static count destructures: `let [small, mid, big] = Group.sorted(by: want)` [r10 40].
- `grid` and `Cell + labels(…)` are adopted (below) [r13 P13]; `ring` is adopted (21.3) [p2 Q2]; `heap_tree` stays in the ports backlog;
  each must remove friction in ≥ 5 ports to come back [r10 47].

## 21.9 Diagnostics (required wording shape)

- Wrong domain: "`deg` is indexed by `V`, but `e` is an `E`. `V` was declared at line 3, `E` at line 4."
- Cut vs element: "`pre` is indexed by `A.Cut`; `x` is an element of `A`. Did you mean `pre[x.after]`
  (prefix through `x`) or `pre[x.before]` (prefix before `x`)?"
- Escaping domain: "`order: [V]V` can't be returned: `V` is local to the test body at line 2."
- `v + 1`: "positions have no `+`. Use `v.next` (`none` at the end), `v.fwd(1)` (clamped), or `V.at(v.ord + 1)`
  (checked)."

## 21.10 Round-3 port rules [p3]

- Positions of one domain are `Ord` (by ordinal). `Bool` is a labeled index space (`[Bool]T`, `(Wall, Bool)`).
- `v.fwd(k)` clamps at the last position, like `v.back(k)`. `.next`/`.prev` are the optional forms.
- One-sided position ranges: `p ..` (p to the last), `.. p` (before p) and `..= p` (the first to p), the same
  grammar as cuts [r16 R6]. On an `Int`-indexed sequence `xs[.. i]` is positions `0..i`.
  `D[c .. c']` (a domain sliced by a cut range) is a `Range<D>`.
- `v.written` is the ordinal as written in the current `io(base:)`, for answers that sum or XOR indices (`uses io`).
- `i.dist(j)` is the shorter distance (`|i.ord - j.ord|` on linear domains); `i.steps_to(j)` is the forward
  distance on rings.
- `ring(labels(N, E, S, W))` is a labeled ring with wrapping `+`/`-`/`walk`, for grid directions.
- A product-domain position is stored as one ordinal in the narrowest unsigned type, with a niche for `none`.
  Unpacking divides by the inner count: a shift for static powers of two, a Barrett constant for runtime
  counts, reported by `cp explain` [r16 R21].
- User structs, impls and generic instantiations that mention a scoped type (a fresh domain, `Mint(mod: p)`)
  are scoped with it. A program-level declaration such as `enum Query<V>` is unscoped; only its instantiations
  at a scoped `V` are scoped.
- `Bool`'s labels are `0`/`1` for I/O, so it reads and prints like any labeled domain.

## 21.11 Grid domains

- **Grid domains** [p1 F3]: `domain Cell = grid(r, c)` is a counted domain over (row, col) with `cell.row`,
  `cell.col`, `cell.neighbours` (4-way; `diag: true` for 8). `domain Node = Cell + labels("S", "T")` adds
  named extra positions, so flow sources need no `V - 2`. `Cell` is fresh and isomorphic to
  `(Cell.Row, Cell.Col)`, associated domains like `A.Cut`, with `cell.row: Cell.Row` and `cell.pair` [r14 T5].
  In a sum domain, cells come first, then labels; `Node.Cut` is a compile error [r14 T23]. `Node(c)` injects a cell and `n.cell: ?Cell` projects back. Under `io(base:)`
  cells convert and labels don't. `read<[Cell]Byte>()` reads r rows of c characters with no separators [r13 P22].

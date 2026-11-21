# Exemplars 11–15 [proposed, against spec p3]

Five problems chosen for what exemplars 1–10 don't cover. Each ends with what it exercises and what it
needed that p3 doesn't have. New items are marked **[new]**.

---

## 11. Monsters (CSES 1194): grid race, multi-source BFS, path output

**Problem.** An r × c grid (≤ 1000) of `.`, `#`, `A` (you) and `M` (monsters). Every step, you and every
monster move one cell. Reach a border cell strictly before any monster can. Print `YES`, the number of
moves and the moves as `LRUD`, or `NO`.

```cp
with io(answers: ("YES", "NO"))
let (r, c): (1..=1000, 1..=1000) = read()
domain Cell = grid(r, c)
let board: [Cell]Byte = read()
let you = Cell.first(where: x => board[x] == 'A')!
let monsters = Cell.filter(x => board[x] == 'M')

// one race: monsters go first, so they win ties, which "strictly before" needs
let race = bfs(from: chain(monsters, [you]), next: x => x.neighbours.filter(y => board[y] != '#'), roots: true)
let exit = Cell.first(where: x => x.on_border and race.root[x] == you)

if let e = exit:
    let path = race.path(to: e)
    let letter = Dir4.map(d => match d: N => 'U', E => 'R', S => 'D', W => 'L')
    print(yes(true))
    print(path.len - 1)
    print(Str(path.pairs().map((a, b) => letter[a.dir_to(b)!])))
else:
    print(yes(false))
```

- **One BFS instead of two.** The usual solution runs a monster BFS, then a player BFS that compares
  distances. Here the race is the algorithm: whoever reaches a cell first owns it, `race.root[x]` says who.
  The tie rule is the queue order of `from:`, which is why it's commented.
- `race.root[x] == you` compares a `?Cell` with a `Cell` structurally, so unreachable cells are just false.
- No `INF`, no `dx`/`dy` arrays, no `-1` parents, no reversing the path.

**Needs [new]:**
- `bfs(from:, next:) -> Search<D>`, generic over any domain: `.dist: [D]?(0..D.count)` (4 B with a niche),
  `.parent: [D]?D`, `.order`, `.path(to:) -> List<D>` (root first), `.tree()`, and `.root: [D]?D` only with
  `roots: true` [r16 R10]. Sources are dequeued in `from:` order and each level keeps its parents' order:
  a guarantee, because Monsters depends on it. It replaces `g.bfs_dist`, `g.bfs_order` and `g.bfs_tree`
  [r16 R11], and it's the helper the minimal-stdlib Split wanted.
- Grid: `cell.on_border`; stdlib `Dir4 = ring(labels(N, E, S, W))` with `cell.step(d) -> ?Cell` and
  `a.dir_to(b) -> ?Dir4`.

---

## 12. Flight Discount (CSES 1195): Dijkstra over a product state

**Problem.** n ≤ 10⁵ cities, m ≤ 2·10⁵ directed flights with prices ≤ 10⁹. One coupon halves (floor) one
flight. Cheapest route from city 1 to city n.

```cp
with io(base: 1)
domain V = read(2..=1e5)
let m: 1..=2e5 = read()
let g = Graph<V>.directed(edges: read<[m]Edge<V, 1..=1e9>>())

type State = (V, Bool)                                // (city, coupon used)
var dist: [State]?Int = none
var heap = Heap<(Int, State)>(order: min)

fn reach(s: State, d: Int):
    if dist[s] min= d: heap.push((d, s))

reach((V.first, false), 0)
while let (d, (u, used)) = heap.pop():
    if d != dist[(u, used)]: continue                  // stale entry
    for (v, w) in g.out(u):
        reach((v, used), d + w)
        if not used: reach((v, true), d + w / 2)

print(dist[(V.last, true)]!)                          // halving never hurts, and a route exists
```

- The state space is a product domain, so `dist` is one dense `[V][Bool]?Int` and a state is a tuple you
  can push, compare and destructure. No `id = 2 * v + used`.
- `?Int` distances: no `1e18` sentinel. `d != dist[s]` is structural, which is exactly the stale check.
- `reach` is a nested function capturing `dist` and `heap`, so the relaxation is written once.

**Needs [new]:**
- **`x min= y` as a condition** yields whether `x` changed. It may only be the *whole* condition of an
  `if`/`while`, like `if let`, so `if a min= b and c` and the `if (x = y)` class can't happen. A `none`
  target always changes; a lifted `none` right side and an equal value don't [r16 R13]. As a statement
  it's unchanged.
- `g.adj(u)` always yields `V`; `g.out(u)` yields `(V, W)` on a weighted graph, so `next: g.adj` works for
  `bfs` on any graph [r16 R12].
- Considered: a generic `shortest(from:, next:)` beside `bfs`. It fits when `next` is a plain adjacency;
  for states with optional moves the loop above is clearer, so it stays a backlog item.

---

## 13. Guess the K-th Zero, easy (CF 1520F1): interactive binary search

**Problem.** A hidden 0/1 array of length n ≤ 2·10⁵ (t = 1). You're given k. Query `? l r` for the sum
of `a[l..=r]`, at most 20 times, then answer `! x`, the position of the k-th zero.

```cp
with io(base: 1)
domain I = read(1..=2e5)
let _: 1..=1 = read()
let k: 1..=I.count = read()
var judge = Wire<Ask<I>>(error_reply: -1, limit: [asks: 20])

fn zeros(upto: I) -> Int:
    let r = ..= upto
    let ones: 0..=I.count = judge.query(.ask(r))
    return r.len - ones

let cut = I.partition_point(x => zeros(upto: x) >= k)
judge.say(.answer(cut.right!))                        // a k-th zero exists, so the cut isn't last
```

- The binary search is `partition_point`: no `lo`/`hi`/`mid`, no `while lo < hi`, no off-by-one in the
  bounds. It makes ⌈log₂(n + 1)⌉ ≤ 18 queries, under the limit of 20, and debug traps on a 21st at this line.
  Its predicate has an effect (a query), so debug doesn't sample it for monotonicity, and `cp explain` says
  so [r16 R1].
- `r.len - ones` counts zeros in a range of positions, so 1-based `l r` never appear: the range prints as
  `first last` in the file's base.

In the same file, never submitted:

```cp
enum Ask<I>: ask(Range<I>), answer(I)
impl Read + Print for Ask<I>:
    fn print(q) = match q:
        ask(r) => "? {r}"
        answer(x) => "! {x}"
    fn read() = match read<Byte>():
        '?' => ask(read())
        '!' => answer(read())
        other => fail("bad token {other}")
impl Limited for Ask<I>:
    type Kind = labels(asks)
    fn kind(q) -> ?Kind = match q:
        ask(_) => asks
        answer(_) => none

struct Case: bits: List<Bool>, k: Int
@gen fn hidden() -> Case:
    domain I = gen(1..=50)
    var bits = gen<[I]Bool>()
    bits[gen<I>()] = false                            // the statement guarantees a k-th zero
    return Case(bits: bits.list(), k: gen(1..=bits.count(where: b => not b)))

@interactor
fn local_judge(test: Case, wire: Wire):
    domain I = test.bits.len
    let bits = I.map(i => test.bits[i.ord])
    print(I.count, 1)
    print(test.k)
    var w = wire.typed<Ask<I>>(limit: [asks: 20])
    loop:
        match w.read():
            ask(r) => print(bits[r].count(where: b => b))
            answer(x) =>
                check(not bits[x] and bits[..= x].count(where: b => not b) == test.k)
                break
```

- The interactor is the naive judge (O(n) per query), which is the point: it's obviously right.
- Both sides share `Ask`'s format and limit; the first real exercise of the p3 `Wire` design.

**Needs [new]:**
- **Interactor signature:** `@interactor fn j(test: T, wire: Wire)` where `T` is whatever the `@gen`
  returns, so a single-test problem takes a `Case` and a multitest one a `List<Case>` (and prints `t`
  itself). With several `@gen`s, pair explicitly: `@interactor(gen: hidden)` [r16 R14]. This replaces the
  fixed `tests: List<Case>` in 4.4.
- `let _: 1..=1 = read()` reads and checks a value you don't need. Already legal by the `_ =` rule; noted
  because `t = 1` lines are common.

---

## 14. Gerald and Giant Chess (CF 559C): counting with obstacles

**Problem.** An h × w board (≤ 10⁵) with n ≤ 2000 black cells. Count monotone (right/down) paths from
the top-left to the bottom-right that avoid black cells, mod 10⁹ + 7.

```cp
type Z = Mint(mod: 1_000_000_007)
let (h, w, n): (1..=1e5, 1..=1e5, 1..=2000) = read()
let black: [n](1..=h, 1..=w) = read()
let f = Z.factorials(h + w)

type Pt = (Int, Int)
fn paths((r1, c1): Pt, (r2, c2): Pt) -> Z = f.binom(r2 - r1 + c2 - c1, r2 - r1)   // 0 unless down-right

let (domain P, pt) = chain(black, [(h, w)]).sorted().numbered()     // the goal sorts last

@memo
fn clean(p: P) -> Z = paths((1, 1), pt[p]) - (..p).map(q => clean(q) * paths(pt[q], pt[p])).sum()

print(clean(P.last))
```

- `clean(p)` = paths to `p` that touch no earlier black cell. The goal joins the black cells as one more
  point, so the answer needs no separate formula.
- No dominance check: `binom` is 0 outside `0 ≤ k ≤ n`, which covers both "above" and "left of" [r12].
- `@memo` over `P` is a dense table; `q` is drawn from `..p`, a range ending before the parameter, so the
  depth is inferred (≤ 2001) [r16 R5]. A `@memo` call counts as pure, so it may sit in a lazy `map` [r16 R2].
- Coordinates stay integer ranges, not a 10¹⁰-cell grid domain: they're numbers in a formula, never indices.

**Needs [new]:**
- Tuple patterns in `fn` parameters, with types (`(r1, c1): Pt`) [r16 R3].
- `chain`, list literals and `if` branches join range types to their hull, so `(h, w)` joins
  `(1..=h, 1..=w)` [r16 R4]. `numbered` accepts any finite sequence.

---

## 15. Password (CF 126B): borders of a string

**Problem.** A string s (≤ 10⁶). Find the longest t that is a prefix, a suffix, and occurs somewhere
else inside s. Print it, or `Just a legend`.

```cp
let s: Str(len: 1..=1e6) = read()
let pi = s.prefix_function()                          // pi[i]: longest proper border of s[..= i]
let inner = pi[.. s.len - 1].max() ?? 0               // longest prefix that recurs ending before the end
if let l = s.borders().first(where: b => b <= inner):
    print(s[.. l])
else:
    print("Just a legend")
```

- Why it's right: a prefix of length `pi[i] ≥ l` ends at `i < n - 1` and starts after 0, so the prefix of
  length `l` occurs strictly inside. `borders()` is longest first.
- `pi[.. s.len - 1].max()` is `?Int` because the slice is empty when `|s| = 1`; `?? 0` says what that means.

**Needs [new]:**
- `s.prefix_function() -> [s.len]0..s.len`, `s.z_function()`, and `s.borders()`: lengths of the proper,
  non-empty borders, longest first [r16 R7]. The cold path is one line:
  `successors(first: pi.last, next: l => if l > 0: pi[l - 1] else: none).take_while(l => l > 0)`.
- `xs[.. i]` on an `Int`-indexed sequence: the half-open one-sided form `.. p` joins `p ..` and `..= p`
  [r16 R6].

---

## What these five found

| # | item | kind |
|---|---|---|
| 1 | generic `bfs(from:, next:) -> Search<D>` replacing `bfs_dist`/`bfs_order`/`bfs_tree` | consolidation |
| 2 | `x min= y` / `max=` as the whole condition of `if`/`while` means "changed?" | small language rule |
| 3 | `@interactor` takes the `@gen` value's type; `@interactor(gen:)` when ambiguous | spec fix (4.4) |
| 4 | grid `on_border`, `Dir4`, `step`, `dir_to` | stdlib |
| 5 | `prefix_function`, `z_function`, `borders` | stdlib |
| 6 | typed tuple patterns in `fn` parameters; `adj` yields `V`, `out` yields `(V, W)` | spec holes |
| 7 | effectful `partition_point` predicates aren't sampled; `@memo` calls are pure | p3 contradictions (r16) |
| 8 | `.. p` one-sided form; range-type hulls; `..p` in depth inference | spec holes (r16) |

Review round 16 (wf/grill-r16.md) found 24 items, all applied above and in the spec. Three were real
contradictions with p3 (R1, R2, R6), so my first claim of "no contradictions" was wrong.

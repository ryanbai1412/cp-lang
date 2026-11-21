# Dream triage against the spec

`scripts/spec-check.sh` checks every dream (`proto/ports/*/dream/*.cp`) against the
`spec/std/` stubs, types only. This file records how each error of each failing dream
was classified and what was done about it.

| | dreams passing | failing | errors |
|---|---|---|---|
| before | 53 / 115 | 62 | 177 |
| after | 88 / 115 | 27 | 48 |

Every remaining error is a checker gap or missing syntax (tables 3 and 4). Rules used:
`decisions.md`, `spec/DREAM-BRIEF.md` and `design/spec/`. `stdlib/` and `crates/` are
untouched.

## 1. Spec gaps (added to `spec/std/`, bodies `= todo`)

Each module was copied from `stdlib/` first; every new signature has a one-line cost doc.

| stub | module | dreams that need it | snippet |
|---|---|---|---|
| `Graph.has(u, v)` | graph | cf2150f | `if not g.has(j, k)` |
| `Graph.reachable(from, skip:)` | graph | cf2115e | `next.reachable(from: V.first, skip: (u, v) => u < p and p < v)` |
| `Graph.hops()` all-pairs BFS | graph | cf2057e2 | `Graph<V>.undirected(edges).hops()` |
| `Adj.transpose` | graph | cf2115b | `let rev = g.transpose()` |
| `complement_components(g)` | graph | agc077c | `for comp in complement_components(forbidden):` |
| `UndirectedMap<V, T>(flip:)` | graph | cf2127h | `UndirectedMap(flip: e => e.transposed)` |
| `Rooted.median(a, b, c)`, `Rooted.adj(v)` | tree | cf2150f | `t.adj(i).pairs` |
| `PotentialDsu.new(op:)` (`+` or `^`) | dsu | cf2022e1 | `PotentialDsu<Line>(op: ^)` |
| `sign(x)` | math | cf2255e2 | `f.path[Sign(sign(c.val))]` |
| `zip` over any two sequences | seq | cf2084c | `zip(couples, zip(I, I.rev))` |
| `toggles(xs, key)` live ranges | seq | cf2069f | `qs.toggles(key: q => (q.graph, minmax(q.u, q.v)))` |
| `iterate(from:, step:)` | seq | cf2066e | `let stuck_points = iterate(...)` |
| `over_time(present, add, answer, rollback)` | timetree | cf2069f | `over_time(present, add, answer: t => ..., rollback: ...)` |
| `Diff.advance(i)` | prefix | cf2124f2 | `dp[c] -= overcount.advance(c)` |
| `Prefix.cyclic_sum(r)` | prefix | cf2089c2 | `pref.cyclic_sum(p - (rem - 1)..=p)` |
| `MonoDeque.push(at: P)`, `drop(at: P)` with domain positions | window | cf2096f | `ends[s.a].push(at: i, val: s.b.r)` |
| `SortedBag.sum(r)`, `SortedSet.sum(r)`, `SortedSet.min_gap(r)` | bag | cf2066e | `unique.sum(..b)`, `unique.min_gap(b..)!` |
| `LinkedList<T>` with splicing | deque | agc079c | `LinkedList.of([v])` |
| `PMultiset<D>` (persistent, `Ord`, `Hash`) | persistent | cf2135f | `var set: [V]PMultiset<Key> = PMultiset.empty` |
| `MeldHeap<T>` | heap | cf2152h2 | `set[kept].meld(set[merged].take())` |
| `MinHeap.new(by:)` | heap | cf2159f | `MinHeap<Seg>(by: s => s.val)` |
| `Poly<T>` | poly | cf2089e | `type P = Poly<Z>`, `a.transposed_mul(b)` |
| `SubproductTree<T>(xs, keep:)` | poly | ucup5s05f | `SubproductTree([Z(u) for u in 0..n], keep: n + 1)` |
| `suffix_tree_edges(s)` | strings | ucup5s05f | `let edges = suffix_tree_edges(s)` |
| `Rng.coin()` | rand | cf2129f2 | `if rng.coin(): ... else: ...` |
| `Rope<D, T>` (reverse, range tag, path query) | lazy | cf2115f1 | `rope.reverse(..=decode_pos(b))`, `rope.query_path(...)` |
| `MergeSeg<D>` (mergeable segment sets) | segtree | cf2115f1 | `MergeSeg.singleton(q)`, `seg.first_alive(id => alive[id])` |
| `monge_path` also returns `edges` (fewest on ties) | dp | cf2183h | `count: path.edges.last!` |

Reused rather than added: `cmp` became `<=>`; `SweepAdd` became `Diff` + `advance`;
`BucketCount` became `Fenwick`; `Pair` became a named tuple; `Dsu(rollback: true)` is plain
`Dsu` (it always has `mark`/`undo_to`); `yes(...)` became `print(..., as: .yes_no)`.

## 2. Dream bugs (fixed in the dream)

| dream | was | now | rule |
|---|---|---|---|
| agc079c | `LinkedList.of(v)`, no `use` | `LinkedList.of([v])` + `use std.deque.LinkedList` | `of` takes a list like `MinHeap.of`, `SortedBag.of`; missing use |
| cf1146h | `for i in V, j in V.after(i), k in ...` | `for i in V for j in ... for k in ...` | one spelling for multi-generator comprehensions |
| cf2150g | `for z in Cnt, o in Cnt`; `rays[z + 1, o - 1]` over `domain Cnt` | nested `for`; `rays: [0..=k, 0..=k]Z` | same; linear domains have no `+`/`-` (21-index-spaces) |
| cf2006d | `Small(..)`/`Big(..)` undeclared; `small[1]` | `enum Side: Small(Level), Big(Level)`; `small[Level.first]` | plain mistake; wrong index space |
| cf2022e1 | `PotentialDsu<Line>(xor)`, no `use` | `PotentialDsu<Line>(op: ^)` + use | options are named; `xor` is spelled `^` |
| cf2039f1 | `grown + {y: 1}`; `above[1]` | `grown + [y: 1]`; `above[G.first]` | map literal is `[k: v]`; wrong index space |
| cf2138f | `{ p1: 1, p2: 2 }`; `fn ladder(mut off_tip: Vec, ...)` called with values | `[p1: 1, p2: 2]`; `var off_tip: Vec` | map literal; a local copy is a `var` parameter, `mut` needs `mut` at the call |
| cf2062g | `fn sort_perm<D>(mut perm: [D]D)` called with `mate` | `var perm: [D]D` | same |
| cf2066d2 | `Time.last(t => a[t] == f)` | `Time.rev.first(t => a[t] == f)` | domains get `.first(pred)`; `.last` is a property (decisions, sequences) |
| cf2066f | `[Seg]Range<PosB.bounds>`; `first_some(x => (x, plan(x)?))` | `[Seg]Range<PosB>`; `first_some(x => plan(x).map(p => (x, p)))` | a cut range is a piece range; `?` cannot leave a fn from a lambda |
| cf2069f | `Pair<V>`, `Dsu(rollback: true)`, hand-rolled time tree | named tuple keyed by `minmax`, `Dsu`, `toggles` + `over_time` | names not in the library |
| cf2089c2 | `domain Pos = 0..n (cyclic)`; `dp[k] = next_row` | `domain Pos = ring(n)`; `dp.row(k) = next_row` | cyclic domains are `ring(n)`; rows are `.row` views |
| cf2096f | `yes(...)` | `print(..., as: .yes_no)` | output formats are `as:` |
| cf2115e, cf2127h | `small[v]`, `a[au]` partial index of a 2D table | `small.row(v)`, `a.row(au)` | rows are lending `.row` views (decision 5.12) |
| cf2124f2 | `SweepAdd<Cut, Z>` | `Diff<Cut, Z>` + `advance` | reuse vocabulary |
| cf2129c3 | `fn ask(inds: Seq<Pos>)` | `fn ask<S: Seq>(inds: S)` | `Seq` is a trait, not a type |
| cf2129e | `BucketCount(0..bit_ceil(n + 1))`; `g.deg[..b]` | `Fenwick<Value, Int>`; `deg = [V] v => g.deg(v)` | reuse vocabulary; `deg` is a method |
| cf2129f1 | `masks.inverse()` of a `[MaxVal]Set<Query>`; `var mask = {}` | a `Map` filled in a loop; `Set()` | `inverse` needs domain values (21-index-spaces); `{}` is not an empty set |
| cf2135f | `sort(cmp: ... then ...)`; `PMultiset<0..n>` | `sorted(by: v => (desc(set[v]), v))`; `domain Key = 0..n` | ordering is `by:` + `desc`; structs take a named domain |
| cf2147f | `cmp(a, b)`; `fn cover(i: Item, by d: Int)`; lazy tree arguments | `a <=> b`; `fn cover(i: Item, by: Int)`; `Action(...)` | no argument labels (14-functions); `LazySegTree` takes an `Action` |
| cf2147h | `gcd(list)` | `fold(gcd, list)` | `gcd` is binary |
| cf2150f | `g.spanning_tree()`; `t.adj[i]` | `Rooted(g, root: V.first)`; `t.adj(i)` | names; `adj` is a method |
| cf2152g | `LazySegTree(Flip(Side), ...)` | `flip_action<Side>(Monoid(Side.empty, Side.join))` | the stock action is `flip_action` |
| cf2157f | `[0; plan.d]`; `[Pos]` table indexed by `Int` | `List.filled(plan.d, 0)`; `fn pos_of(idx: Int) -> Pos` | not the spelling; wrong index space |
| cf2165e | `std.containers.SortedBag`, `from`, `insert`, `max` | `std.bag`, `of`, `add`, `last` | library vocabulary |
| cf2165f | `latest.fold(a[j] <..a[i])` | `latest.fold(a[j].r..a[i].l)` | `<..` is not a range form; cut ranges convert to pieces |
| cf2178h | `Zmod(m)`; `Move.all` over a payload enum; `fixed_cost_paths` positional | `ring(m)`; explicit move list; `from: [...]`, `cost:`, `step:`, `dist.get(...)` | only payload-free enums are domains (11-types); signature |
| cf2178i | `Vec2<1..=100>`; missing use | `Pt`; `use std.graph.two_color` | library name; missing use |
| cf2180f1 | `fn stops<domain I, domain J>()`; `struct Shut(floor: Bool, wall: Bool)` as an index | `fn stops<I, J>()`; `domain Shut = (floor: Bool, wall: Bool)`, `[Shut] s => ...` | domain parameters are plain type parameters (12-scoped-types); index by a product domain |
| cf2183h | `aliens(k, solve: ...)`; `monge_path(list, cost: ...)` | `aliens(k, lambdas: ..., solve: penalized)`; `monge_path(nodes.len, w: ...)` | signatures |
| cf2222f, cf2255e2, dmopc20c7p5, cf2147h, cf2157f | missing `use` (`DsuOf`, `MaxSub`, `DualSegTree`, `sign`, `max_closure`, `gomory_hu`, `choose`) | added | missing use |
| cf2237h | `fn take(need: Int, before t: Time)`; `(t, need: r.len)` | `before: Time`; `(t: t, need: r.len)` | no argument labels; a tuple is all named or all positional |
| cf2255e2 | `DualSegTree<I>([I] ..., id: ...)`; `seen * k`; `(let a = ..., b = ...; ...)`; `_ => .empty` | `a: ...`; `seen.scale(k)`; local `fn path`; `MaxSub.empty()` | named arg; `MaxSub` scales; no let-expressions; leading dot only for enums (31-interaction) |
| cf2262b | `tops.range(c.l..stop).len` | `tops.count_range(c.l, stop)` | costs are visible: counting a tree range is O(log n + k) |

## 3. Checker gaps (dream is right; not changed)

| dream | snippet | why the dream is right | what the checker does |
|---|---|---|---|
| cf2089c2, cf2178h | `domain Pos = ring(n)`; `dp[k, p - 1]` | 21-index-spaces: `ring(n)` is a cyclic domain with wrapping `+`/`-` | no `ring`; `p - 1` is `?Pos` |
| cf2084e | `tally(Pos, Pos, 1)` | 21-index-spaces: a domain converts to its full range | wants `Range<Pos>`, rejects domain `Pos` |
| cf2021d | `right_ends(-pref.rev, end?.rev)` | `.rev` of a `[D]T` is a `[D]T` view (`pref.rev` checks); `?.` maps over the optional | `end?.rev` becomes `List<Int>` |
| cf2062h | `var dp: [Row.spans][Subset<Col>]Tally = (ways: 0, boxes: 0)` | integer literals take the expected `Mod` type (they do outside tuples) | tuple literal stays `(Int, Int)` |
| cf2180f2 | `use cf2180f1.{Z, stops}` | a dream may import its sibling (F1 and F2 share code) | `spec-check.sh` checks each file alone: no module `cf2180f1` (tool gap) |
| cf2129f2 | `ask<[min(probes.len, TYPE3_LIMIT)]Val>(3, ...)` | DREAM-BRIEF: generic calls are `f<T>(...)` | `<` after a name parses as less-than when the type argument holds a call |

## 4. Syntax the parser lacks (not implemented)

| form | dreams | covered by | recommendation |
|---|---|---|---|
| contexts `with io(base: 1)` | cf2062e1, cf2062e2, cf2150f, dmopc20c7p5 | 17-contexts §17.1 | implement as specified |
| `with conn.checkpoint():` (scoped rollback) | cf2222f | 17-contexts (contexts as values) | implement with contexts; same block form |
| sum domains `domain Line = Row(1..=n) \| Col(1..=m)` | cf2022e1 | 21-index-spaces (sum domains), decisions (noted as the general fix) | implement; it is the core fix for "extra vertices" |
| tuple-pattern parameters `fn step(mv: Move, (r, s): State)` | cf2178h | 14-functions (`fn paths((r1, c1): Pt, ...)`) | implement as specified |
| lowercase variant patterns `decr(l, r: leaf) =>` | cf2187f1, cf2187f2 | 11-types §11.2 | implement as specified, with the `r: leaf` sub-pattern |
| `var` fields, fields typed by their default | cf2084h | not in the design | decide: struct fields are already mutable through `var` bindings; prefer a plain field with a default |
| `Str of "01?"` alphabet-restricted strings | cf2187d | not in the design | decide; it gives validation and 2-bit storage, worth a decision |
| set comprehension `{ f(i) for i in xs }` | cf2187d, cf2178i | not in the design | add: list comprehensions already exist, sets are the same rule |
| filtered domains `domain A = V if side[v] == small` | cf2178i | 21-index-spaces offers `xs.numbered()` instead | prefer the decided `numbered()`; or decide sugar for it |
| array type with read length `[read(1..=2e5)]T` | cf2066c | not in the design | keep `domain I = read(...)`; reject in the dream brief |
| map with default `[0: 1] default 0` | cf2066c | not in the design (brief: `m[k] += 1` inserts the default) | add a `default:` named option to `Map` rather than syntax |
| char-literal union type `('+' \| '-', Vol)` | cf2066e | not in the design | decide; a small enum read by label covers it today |
| one-line match arms `match x: A => .., B => ..` / `1 => ..; 2 => ..` | cf2006d, cf2255e2 | 10-syntax (one-line bodies after `:`) does not cover several arms | decide one separator (`,`) and allow it |
| bare operator values before `,` `(count: +, taken: or)` | cf1984e | decision 3.1 (bare operators are values) | fix the parser to accept an operator value wherever an expression ends; decide whether `or` is a value |
| `for (var dies, val) in xs` | cf2002e | decisions 15.12 covers `var (a, b) = ...`, not per-name `var` | allow `var` per pattern name, or spell it `for var (a, b) in` |
| compound assignment as a lambda body `(v, a) => cnt[..] -= 1` | cf2245d2 | not in the design | allow statement bodies for unit lambdas |
| leading-dot continuation line `\n    .map(...)` | cf2129c3 | not in the design | allow a line starting with `.` to continue the expression |
| lazy tables `lazy[0..B] j => f(j)` | cf2159e | not in the design | decide; `@memo fn` covers it today |

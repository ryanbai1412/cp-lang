# CP Lang — One-Page Contest Cheat Sheet

The contest essentials. Every CP block is verified with `cpl check`, `cpl build`, and C++17; the prelude needs no imports.

## 1. Bindings, numbers, and operators

```cp
let n = 10                         // immutable binding
var answer = 0                     // mutable binding (use `var` for mutable variables)
const MOD = 998_244_353             // compile-time constant
answer += n
print(answer)
```

- `Int`: symmetric signed 64-bit; `U64`: wrapping unsigned 64-bit; `I128`: signed 128-bit; `F64`: double.
- `0..=1_000_000_000` is a checked range type; arrays pack bounded values tightly.
- `1e9` is an `Int`; `1.0` and `1e-9` are `F64`. Hex `0xFF`, binary `0b1010`.
- `/` and `%` are Euclidean: the remainder is nonnegative, even with negative operands.
- Power `**`; bitwise `& | ^ ~ << >>`; logic `and or not`; membership `in`, `not in`.
- Updates `+= -= *= /= %=`; `x min= y`, `x max= y`; any binary function supports `x f= y`.
- Cast: `F64(x)`, `Int(ch)`, `Char(i)`; Int to Str: `"{x}"` (no `.to_str()`); `abs(x)` is a free function (no `.abs()`). For floats choose `floor`, `ceil`, `round`, or `trunc`.
- Print floats with precision: `print("{x:.6f}")`. NaN/Inf trap locally; compare floats with `abs(a - b) < eps`.
- Indentation defines blocks; `//` comments; `;` separates inline statements, never trails a line.

## 2. Optionals and relaxation

`?T` means a value or `none`, not an `INF`/`-1` sentinel. Common optionals are niche-packed.

```cp
domain V = 0..3
var best: [V]?Int = none
let (u, v, w) = (V(0), V(1), 7)
best[u] = 0
let via = best[u] + w               // none absorbs +
if best[v] min= via: print("improved")
if best[v] min= via: print("not reached: equal candidate")
if let d = best[v]: print(d)
print(best[V(2)] ?? -1)             // fallback only on none
```

- `min=`/`max=` return `Bool`: true exactly when the destination changes. Ties return false.
- A `none` candidate is skipped; a present candidate improves an empty destination.
- Arithmetic (`+ - * / % **`) and bitwise (`& | ^ << >> ~`) ops lift over `?Int`/`?F64`; `+=`/`-=` do not.
- `opt!` asserts present; `opt ?? return none` exits early; `if let x = opt` unwraps safely; `if let x = opt, x > 0:` guards.
- `min`/`max` of a possibly empty iterable return `?T`; `sum` is for non-optional numbers.
- `?T` and `Mod<P>` do not implement `Ord`; sort with an explicit non-optional key.
- `m[k] ??= init` lazily initializes a missing map entry; map updates use operator identities.

## 3. Domains, arrays, boundaries, and grids

```cp
domain V = 1..=4                    // nominal index type, distinct from Int
let a: [V]Int = [V] v => v.to_idx + 10
print(a[V.first], a[V.last])        // domain endpoints are indices
print(a.first, a.last)              // array endpoints are ?Int values
var count: [0..=9]Int = 0           // structural array, ordinary Int indices
count[3] += 1
count.fill(0)
```

- `[V]T` uses `v: V`; `[0..n]T` uses `i: Int`. `domain V = n` is `0..n`; `V.count` / `V.K` is the size; `V.first`/`V.last` need nonempty `V`; `v.to_idx` is 0-based.
- `var dp: [0..n][0..4]Int = 0` fills nested arrays. Array `.first`/`.last` return `?T`; unwrap nonempty ends with `!`.
- `v + k` / `v - k` return `?V`; `b - a` returns `Int`; `v.cyc(k)` wraps; `for v in V by k` steps a domain.
- `V.bounds` has `V.K + 1` gaps; `v.l`/`v.r`, `v.l(k)`/`v.r(k)` pick boundaries; `b.left`/`b.right` give `?V`; range ops accept `..v`, `..=v`, `v..`.
- `[Row, Col]T` indexes `g[r, c]`; `g[r, ..]`/`g[.., c]` give `[D]T`; `p.step(d)` over `Dir4`/`Dir8` gives `?(Row, Col)`; `use std.grid.{adj4}`.
- Compression: `xs = compress(all)`, fresh domain `xs.D`, `xs.at(x)`, inverse `xs[id]`.

## 4. Control flow and ranges

```cp
let x = 7
let sign = if x < 0: -1 else: 1
if x < 0: print("negative")
elif x == 0: print("zero")           // elif, never else if
else: print("positive")
var (a, b) = (48, 18)
while b != 0: (a, b) = (b, a % b)   // both RHS values before either write
var best = 0, ways = 1              // two bindings on one line
repeat 3: ways *= 2                 // for _ in 0..3
loop:                               // while true
    best += 1
    if best == 2: break
for i in (0..6).step(2).rev: print(i)
for i in 3..:                       // open-ended Int range; stop explicitly
    if i == 6: break
    print(i)
```

- Half-open `l..r`, inclusive `l..=r`; arithmetic binds tighter than ranges.
- Finite ranges provide `.first`, `.last`, `.mid` (all optional), `.len`, `.is_empty`, `.rev`.
- `for x in xs if predicate:` filters; `break 'outer` targets a labeled enclosing loop; `while let x = q.pop(), x > 0:` drops the `x` failing its guard.
- Open-ended `Int` ranges need a stopping condition; do not collect an unbounded iteration.

## 5. Functions, closures, and ownership

```cp
fn twice<T>(x: T) -> T = x + x
fn bump(mut x: Int): x += 1
var n = 3
bump(mut n)                         // mut on parameter and call site
let xs = [3, 1, 2]
print(sorted(xs, by: x => -x))       // one closure spelling
print(twice(n))
```

- Function types: `T -> Bool`, `(T, T) -> T`; generic calls can specify `f::<T>(args)`.
- Nested functions capture outer bindings; use explicit `mut` parameters for borrowed mutation.
- Small values copy; heap values move on last use. `.copy()` clones; `.take()` moves out.
- `swap(mut a, mut b)` is O(1); no stored or returned borrows.
- Receiver/index expressions and the RHS finish before an update borrows its destination.

## 6. Records, structs, and enums

```cp
type Edge = (to: Int, weight: Int)   // structural record, e.to or tuple destructuring
struct Point(x: Int, y: Int)
enum Op:
    Add(delta: Int)
    Query
enum Dir: N, E, S, W                 // one line; `|` also separates
let e: Edge = (2, 7)
let op = Add(e.weight)
match op:
    Add(delta) => print(delta)
    Query => print("query")
```

`Eq`, `Ord`, `Hash`, `Show`, `Read`, `Index`, and `Seq` derive when fields support them.
Domains, enums, tuples, `Bool`, and `Subset<D>` provide typed index spaces.
Use `use lib.module.{Name}` for a pinned local library; `pub` exports a declaration.

## 7. ASCII strings

```cp
let s = "123ab"
let ch = s[0]                       // Char, one ASCII byte
print(ch - '0', s[1..3])            // O(1) indexing; slicing copies
var out = ""
out.push('a')
for _ in 0..3: out += "bc"
print(out)
print(s.starts_with("12"), s.find("ab"), 'a'.upper) // YES 3 A
```

`Str` is `0..s.len` ASCII bytes: `s[..k]`, `s[k..]`, `.len`, `.starts_with`, `.ends_with`, `.find`/`.rfind` (Char or Str) `-> ?Int`, `sub in s`, `s.find_all(p)`, `Char.lower`/`.upper`/`.digit`. For nominal positions: `let s: [domain S]Char = read()`.

## 8. Collections and sequence construction

```cp
let xs = [1, 2]
var q = Deque.of(xs)                // front-to-back, O(n)
while let u = q.pop_front(): print(u) // 1, 2; pops return ?T, none when empty
print(xs.resized(4, 0))             // 1 2 0 0; xs is unchanged
print(resized(xs, 1, 0))            // 1; same helper, truncating instead of padding
```

- Containers share `.len`, `.is_empty`, `.clear()`, `.take()`; `List`/`Deque` can start as `[]`.
- `List<T>`: `.push`, `.pop -> ?T`, `.insert`, `.remove`, `.swap`, `.first`/`.last -> ?T`, `xs[l..r]`.
- `Deque<T>`: `.push`/`.pop -> ?T` at back (no `.push_back`), `.push_front`/`.pop_front -> ?T` at front, `q[i]`; all O(1).
- `MinHeap<T>`/`MaxHeap<T>`: `.push`, `.pop -> ?T`, `.peek -> ?T`; `desc(x)` flips one key field: `MinHeap<(Desc<Int>, V)>`.
- `Set<K>`/`SortedBag<K>`: `.add`, `.remove`, membership, `.first_ge`, `.last_le`, `.first`, `.last`, `.pop_first()`, `.pop_last()`; bag keeps duplicates.
- `Map<K,V>`/`HashMap<K,V>`: direct `m[k]` traps if absent; `.get`, `.remove` return optionals; `.get_or_insert(k, v)` returns the stored value; `.first`, `.pop_first()` give `?(K, V)`.
- `Bitset<D>`: `.add`, `.remove`, `.toggle`, set operations, `.len` (popcount), `.shifted(k)`.
- `Subset<D>`: small bitmask indices; `.empty`, `.full`, `.with(v)`, `.without(v)`, `s = s.set(v, on)` (returns a new subset), `for sub in s.subsets`, `for v in s`.
- Comprehensions `[f(x) for x in xs if ok(x)]`, `[b for v in V if let b = boss[v] and b != v]` (`and`, not `,`); `xs += ys`; `for b in Bool`; `sort(mut xs)`, `sorted(xs, by: desc)`, `sorted(0..n, by: f)`.
- `xs.windows(k)` yields each length-`k` window; `zip(xs, ys)` pairs up to the shorter list.
- Folds take a monoid such as `+`: `fold(+, xs)`, `scan(xs, +)`, `xs.excl_fold(+)` yields `(x, rest)`; `runs`, `runs_by(xs, key)` yields `(key, items)`, `prefix`, `compress`.

## 9. Search and modular arithmetic

```cp
print(bsearch_first(0..=10, x => x * x >= 20)) // 5; false then true
print(bsearch_last(0..=10, x => x * x <= 20))  // 4; true then false
type Z = Mod<7>
var z: Z = 0                        // literals coerce from the expected type
z += 1
z -= 2
z *= 3
let one: Z = 1
print(z, one, Z.P)                  // 4 1 7; numeric type parameter access
```

- Binary search returns `?I`; no hit is `none`. Floating ranges take `steps: 80`.
- Sorted sequence `.first_ge`/`.last_le` return indices; sets return keys; maps return pairs.
- `Mod<P>` supports `+ - * / **`, `.pow`, `.inv` (or `.inv()`), `.try_inv`; `P` can be constant, runtime-frozen or above 2^31.
- `Comb<P>()`: `.fact`, `.inv_fact`, `.choose`, `.arrange`; number theory: `gcd`, `lcm`, `ceil_div`, `Sieve`.
- Randomness: `rand(lo, hi)`, `shuffle(mut xs)`, seeded `Rng.new(seed).int(lo, hi)` (`use std.rand.{Rng}`); geometry: `Pt`.

## 10. Graphs and rooted trees

```cp
domain V = 0..4
let edges = [(V(0), V(1)), (V(1), V(2)), (V(1), V(3))]
let g = Graph.directed(edges)
print(bfs(g, from: [V.first]))       // optional distances; multiple sources allowed
print(peel(g), topo(g))
let t = Rooted(Graph.undirected(edges), V.first)
let down: [V]Int = 1
for (c, rest) in t.excl_fold(V(1), *, c => down[c] + 1):
    print(c, rest)                  // sibling product excluding child c
```

- `peel` returns Kahn's removal order; `topo` a full order or `none` on a cycle; also `scc`, `condense`.
- A neighbour lambda works anywhere a graph does; its domain is what it indexes: `peel(v => adj[v])`.
- Grid cells are vertices: `bfs(p => [q for q in p.adj4 if not wall[q]], [(R(0), C(0))])` (`use std.grid.{adj4}`).
- Unpeeled vertices can be on a cycle **or reachable from one**; not all are cycle vertices.
- `t.excl_fold(v, m, f)` yields `(child, fold over other children)` in O(degree(v)), no inverse needed.
- `t.reroot(max, (h, c, p) => h + 1, (h, v) => max(h, 0))` is `t.up`'s DP from every root; `Centroids<V>(g).ancestors(v)` yields `(c, dist)`.
- `t.Pre`, `t.pos(v)`, `t.sub(v)` support subtrees; `.lca`, `.dist`, `.fold_path` support paths.
- `a.e` (edge index) needs a graph built from `[E](u: V, v: V)` records: `dijkstra(g, from: [(s, 0)], w: a => wt[a.e])`.
- Flow: `FlowGraph<V>()`, `.add(u, v, cap)`, `.max_flow(s, t)` over numeric vertex domains.

## 11. Range structures and named algebra fields

```cp
domain V = 0..4
let a: [V]Int = 1
let m = Monoid<Int>(id: 0, op: (x, y) => x + y)
let add = Action<Int, Int>(
    id: 0,
    compose: (new_f, old_f) => new_f + old_f,
    apply: (f, value, span) => value + f * span,
    over: m,
)
var st = LazySegTree<V, Int>(add, a)
st.apply(V.first..=V.last, 2)
print(st.all)                       // 12
```

`id` is the identity; `compose(new_f, old_f)` applies old first; `span` counts leaves. `SegTree<D,T>(+, a)` takes operators or a `Monoid`; lazy trees take `AddSum`/actions; `.fold(..)` / `l..` / `..=r` work.
`Fenwick`: `.add`/`.sum`/`.kth(k) -> ?D`; `SparseTable`: nonempty `.fold`; segment trees: `.set`/`.fold`/`.all`.

## 12. Input, memoization, and testing

```cp
@!multitest                        // reads t; fresh state and memo cache per case
let n = read(1..=20)
domain V = 1..=n
let a: [V]Int = read()
@memo
fn pref(v: V) -> Int:
    if v == V.first: return a[v]
    pref((v - 1)!) + a[v]
print(pref(V.last))
debug(n, a)                        // local stderr; stripped on judge
```

- `read()` takes the type from the binding (`let k: Int = read()`, never `read(Int)` or untyped `let k = read()`) or a range (`let n = read(1..=20)`). `read(none: 0)` maps `0` to `none`; `print(x, end: " ")` sets the suffix.
- Co-locate `@brute`, `@gen`, `@checker` for `cpl stress`; `@interactor`/`@grader` for judge protocols; `ask("?", x)` prints, flushes, reads.
- `cpl check`, `cpl run`, `cpl build`, `cpl stress`, `cpl test`, `cpl doc`, `cpl lsp`. See [reference](types-and-control.html), [stdlib](stdlib.html), [course](course.html).

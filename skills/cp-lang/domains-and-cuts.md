# Domains, Positions, and Cuts

A `domain` declares a finite index space as a distinct type. Arrays indexed by
that domain cannot be indexed with raw `Int` or with another domain (`E0301`).

## Declaring Domains and Arrays

```cp
let n = 4
domain V = 1..=n
var a = [V] v => Int(v) * 10
a[V(2)] += 5
print(a[V(1)], a[V(2)], a[V(4)])
// output: 10 25 40
```

Key domain properties and conversions:

- `V.count` (or `V.len`): number of positions in `V`
- `V.first`, `V.last`: `?V` (`None` if empty)
- `V(x)`: convert `Int` to `V` (traps with `R0102` if out of range)
- `V.at(x)`: safe conversion returning `?V`
- `Int(v)`: convert `v: V` back to its integer value
- `v.ord` / `v.to_idx`: 0-based index `0..V.count`
- `V.nth(k)`: `k`-th position (0-based)

Mixing domains is a compile-time error:

```cp error(E0301)
domain A = 0..3
domain B = 0..3
var x: [A]Int = 0
print(x[B(0)])
```

## Positions vs Cuts

- A **position** `v: V` names one element. A **cut** names a boundary between
  positions; `V.cuts` is the cut domain (`V.count + 1` cuts), and `Int(c)` is
  the number of positions before `c`.
- The one rule: `.l`/`.r` switch kind, `.prev`/`.next` keep it.

| receiver | same kind | other kind |
|---|---|---|
| position `v` | `v.prev`, `v.next` → `?V` | `v.l`, `v.r` → its two cuts |
| cut `c` | `c.prev`, `c.next` → `?V.cuts` | `c.l`, `c.r` → neighbouring positions, `?V` |

- `v.next(k)`/`v.prev(k)` step by `k`. `v.l..v.r` is exactly `v`.
- `v.after` / `v.before` are the positions after / before `v` (`v.r..`, `..v.l`).
- `v + 1` is an `Int`. Back into the domain: `v.next`, `V(x)` (traps outside),
  `V.at(x)` (`?V`), or `if let V(j) = i - 10:`. `Int(v)` gives the integer.
- Ranges: `l..=r` and `l..r` of positions; `a..b` of cuts. Prefix structures
  take ranges or cuts: `p.fold(l..=r)`, `bit.prefix(v.r)`.

```cp
domain V = 1..=3
for v in V:
    print(Int(v.l), Int(v.r), v.next, end: "; ")
print()
for j in V(1).after:
    print(j, end: " ")
print()
// output: 0 1 2; 1 2 3; 2 3 none; 
// output: 2 3 
```

Cuts prevent off-by-one bugs in prefix sums, DP on prefixes, and Fenwick/SegTree
binary searches:

```cp
domain I = 0..4
let a = [I] i => Int(i) + 1
let pref = Prefix(a)
print(pref.fold(I(1)..=I(3)), pref.all)
// output: 9 10
```

## 2D Arrays, Unordered Pairs, and Subsets

- Multi-dimensional arrays use comma-separated domains: `var dp: [R, C]Int = 0`
  and index as `dp[r, c]`.
- `Pair<V>` represents an unordered pair of distinct vertices (`Pair.new(u, v)`
  normalizes `lo < hi`).
- `Subset<B>` stores a subset of a small domain (`|B| <= 64`) in one `U64`
  word for bitmask DP:

```cp
domain B = 0..4
let s = Subset<B>.single(B(1)).with(B(3))
print(s.len, yes(B(1) in s), yes(B(2) in s))
// output: 2 YES NO
```

## Binary Search on Ranges

Use `.first_true(pred)` or `.last_true(pred)` on integer or domain ranges
instead of hand-writing `while lo < hi`:

```cp
let ans = (0..=1_000_000).first_true(x => x * x >= 50)!
let floor_sqrt = (0..=1_000_000).last_true(x => x * x <= 50)!
print(ans, floor_sqrt)
// output: 8 7
```

Both return `?T` (`None` when no value in the range satisfies `pred`).

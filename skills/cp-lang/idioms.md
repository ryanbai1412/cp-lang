# Idiomatic CP Lang: Worked Before/After Examples

Write CP Lang directly around domains, optionals, and stdlib abstractions
rather than translating C++ line by line.

## 1. Shortest Path Without `INF` Sentinels

**C++ habit:** `vector<long long> dist(n + 1, 1e18);` and manual `-1` checks.

**Idiomatic CP Lang:** `domain V = 1..=n`, `Graph.undirected`, `bfs`, and
`print(..., none: -1)`:

```cp
domain V = 1..=4
let edges = [(V(1), V(2)), (V(2), V(3))]
let g = Graph.undirected(edges)
let res = bfs(g, from: [V(1)])
print(res.dist, none: -1)
// output: 0 1 2 -1
```

## 2. Optional DP With `min=` and Arithmetic Lifting

**C++ habit:** `dp[i] = INF`, guard `if (dp[i] < INF)` before adding `cost`,
then check `dp[n] == INF ? -1 : dp[n]`.

**Idiomatic CP Lang:** `?Int` lifts `+` through `None`, and `min=` ignores
`None`:

```cp
let coins = [3, 4]
let target = 10
domain S = 0..=target
var dp: [S]?Int = None
dp[S(0)] = 0
for s in S:
    for c in coins:
        if let Some(nxt) = S.at(Int(s) + c):
            dp[nxt] min= dp[s] + 1
print(dp[S(target)], none: -1)
// output: 3
```

## 3. Binary Search on the Answer

**C++ habit:** `while (lo < hi) { mid = lo + (hi - lo) / 2; ... }` with
off-by-one risk on `lo = mid` vs `hi = mid`.

**Idiomatic CP Lang:** `(lo..=hi).first_true(pred)` or `.last_true(pred)`:

```cp
let weights = [3, 2, 2, 4, 1, 4]
let days = 3

fn fits(cap: Int) -> Bool:
    var used = 1
    var cur = 0
    for w in weights:
        if cur + w > cap:
            used += 1
            cur = 0
        cur += w
    used <= days

let lo = weights.max()!
let hi = weights.sum()
let ans = (lo..=hi).first_true(cap => fits(cap))!
print(ans)
// output: 6
```

## 4. Range Queries With Domain-Typed `SegTree`

**C++ habit:** 0-based/1-based `seg[4 * n]` with manual `push`/`pull`.

**Idiomatic CP Lang:** `SegTree<D, T>.of(op, init_fn)` over a domain `D`:

```cp
domain I = 0..5
let vals = [2, 5, 1, 4, 3]
var st = SegTree<I, Int>.of(+, i => vals[i.to_idx])
st.set(I(2), 10)
print(st.fold(I(1)..=I(3)), st.all)
// output: 19 24
```

## 5. Sorting by Key and Counting Inversions

**C++ habit:** custom comparator lambdas, `std::pair` hacks, hand-rolled merge
sort for inversions.

**Idiomatic CP Lang:** `by:` named argument, `desc(...)`, and stdlib
`inversions(...)`:

```cp
let xs = [3, 1, 4, 1, 5]
print(xs.sorted(by: x => desc(x)))
print(inversions(xs))
// output: 5 4 3 1 1
// output: 3
```

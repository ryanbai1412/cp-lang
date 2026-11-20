In C++, dynamic programming and shortest paths constantly fight sentinel constants (`1e18`, `INT_MAX / 2`, `-1`): add two `INF` values and you overflow into negative numbers; forget a base-case check and a dead state leaks into the answer. CP Lang 0.2.2 eliminates sentinels with **Tropical Optionals** (`?T`, `min=`, `max=`, `??=`) and pairs them with **zero-boilerplate `@memo` DP**.

## Tropical Optionals: `?T`, `min=`, and `max=`

An optional `?T` holds either any value of type `T` or `none` (so `?Int` represents every 64-bit `Int` plus `none`). The compiler can optimize optional storage layout when it proves a niche or sentinel is safe, rather than reserving `Int.MIN` or promising universal zero overhead.

Three rules make DP transitions one line with no `INF` constant:

1. **Additive ops absorb `none`.** On `?Int` and `?F64`, `+`, `-`, and unary `-` return `none` if either operand is `none`.
2. **`min=` and `max=` skip `none`.** Updating `dp[s] min= cand` does nothing when `cand` is `none`, and stores `cand` when `dp[s]` is `none`. `none` acts as the tropical identity (`+inf` for `min=`, `-inf` for `max=`).
3. **`??` supplies the final fallback.** `print(dp[x] ?? -1)` prints `-1` when state `x` was never reached.

Here is minimum-coin change in 9 lines—compiling and running directly in WebAssembly:

```example tropical_dp
```

When `dp[s - c]` is `none`, `dp[s - c] + 1` evaluates to `none`, and `dp[s] min= none` leaves `dp[s]` untouched. No sentinel arithmetic, no `if dp[s - c] != INF` guard.

```exercise dp-tropical-coins
```

## Lazy initialization with `??=`

Use `x ??= expr` to initialize an optional place only if it is currently `none` (and leave an existing value intact). Combined with `max=`, one pass tracks both the largest even number and the first negative number without boolean flags:

```example optional_coalesce
```

In arena and adjacency code, `(adj[k] ??= []).push(x)` initializes a missing bucket lazily and lends it mutably in one expression.

```exercise dp-optional-coalesce
```

## Tropical guardrails: what never lifts

To prevent silent bugs, only `+`, `-`, unary `-`, `min`/`max`, and `min=`/`max=` lift over `?T`:

- `*`, `/`, `%`, bitwise ops, `+=`, `-=`, and `sum(...)` **never** lift over `?T`. Writing `x += 2` on `var x: ?Int` is a compile error (`E0309`)—unwrap or provide a fallback first.
- `?T` does **not** implement `Ord`, so `<`, `<=`, `>`, `>=` never silently order `none` before or after real values. (`==` and `!=` do auto-lift `T` to `?T`.)
- When `none` means *failure/poison* and every branch must succeed (not tropical identity), check explicitly with `if let`:

```cp CP Lang 0.2.2 · fragment
if let hit = solve(child):
    best min= hit
else:
    best = none
    break
```

```exercise dp-tropical-guardrails
```

## Cached recursion with `@memo`

Attach `@memo` to a pure recursive function over `Index` or range parameters to turn an exponential recurrence into a cached DP table automatically:

```example memo_dp
```

`ways(35)` executes instantaneously in WebAssembly — without `@memo`, naive recursion would perform tens of millions of calls. The compiler lowers `@memo` into a real cache:

- When parameters are bounded `Index` or range types fitting within `2^24` states, `@memo` allocates a flat dense table; larger state spaces switch automatically to a salted hash map.
- A `@memo fn` declared below `@!multitest` or nested inside a loop automatically clears its cache on every test case or iteration — no stale-table Wrong Answers across test cases.
- `@memo` functions can capture immutable outer `let` bindings directly.
- `@memo(into: dp)` keeps the answers in a table you can see: `dp` is a `var dp: [I, J]?T`, one cell per choice of arguments and `none` where the function has not been asked yet. It is always a dense table whatever its size, never a hash map, and you can fill cells ahead of a call, read them afterwards, or clear them:

```cp
let n: 1..=3000 = read()
let a: [0..n]1..=1e9 = read()

@memo(into: dp)
fn best(l: 0..=n, r: 0..=n) -> Int:
    if l >= r: return 0
    max(a[l] - best(l + 1, r), a[r - 1] - best(l, r - 1))

print(best(0, n), count(dp[l, r] != none for l in 0..=n for r in 0..=n))
```

```exercise dp-memo-ways
```

```exercise dp-memo-reset
```

## Bitmask DP with `Subset<D>` and `Bitset<D>`

For exponential subset DP (`n <= 25`), `Subset<D>` wraps a bitmask as a typed `Index` so you never write `1 << i` or `(mask >> i) & 1` by hand:

```cp CP Lang 0.2.2
domain V = 0..n
var dp: [Subset<V>, V]?Int = none
dp[Subset<V>.single(V.first), V.first] = 0

for mask in Subset<V>:
    for u in mask:
        for v in mask.complement:
            dp[mask + v, v] min= dp[mask, u] + cost[u, v]
```

Submask iteration is equally direct: `for sub in mask.subsets:` visits every subset of `mask` in `O(3^n)` total across all masks. For larger bit-parallel DP (such as subset-sum reachability up to `10^5`), `Bitset<D>` packs bits into 64-bit machine words with `|`, `&`, `^`, `<<`, `>>`, `.count`, and `.first_set`.

Next, [algebraic segment trees and `Rooted`](course-algebra.html) replaces ad-hoc tree boilerplate with `Fenwick`, `Monoid(id, op)`, and `Action(id, compose, apply, over)`.

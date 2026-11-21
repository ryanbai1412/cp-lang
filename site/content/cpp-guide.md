If you already solve contest problems in C++17, you already know every algorithm you need. This guide shows how to **rethink** C++17 contest solutions into idiomatic CP Lang — at the gold standard of `design/EXEMPLARS-PORTS.md` — where the editorial invariant reads top-to-bottom, each bound and modulus is written once, and `cpl build` emits fast, judge-ready C++17.

## Rethink, don't translate

Compare how a Codeforces Div2 C (`cf2069c`, Beautiful Sequence: count `1 2…2 3` subsequences mod `998244353`) reads in C++17 vs CP Lang:

```cpp C++17
int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(nullptr);
    int T; cin >> T;
    while (T--) {
        int N; cin >> N;
        using num = modnum<998244353>;
        num dp = 0, ans = 0, empty = 0;
        for (int i = 0; i < N; i++) {
            int a; cin >> a;
            if (a == 1) { dp++; empty++; }
            else if (a == 2) { dp += dp; }
            else if (a == 3) { ans += dp - empty; }
        }
        cout << ans << '\n';
    }
}
```

```cp CP Lang
// A beautiful subsequence is 1 2...2 3 with at least one 2.
// Track subsequences ending in "1" and those ending in "1 2+".
type Z = Mod<998_244_353>

let xs: List<Int> = [1, 2, 1, 2, 3]
var ones: Z = 0
var open: Z = 0
var ans: Z = 0
for a in xs:
    match a:
        1 => ones += 1
        2 => open = 2 * open + ones
        3 => ans += open
        _ => continue
print(ans)
```

Notice what disappeared: `ios_base::sync_with_stdio`, manual `while (T--)` (replaced by `@!multitest` at file top), `#define int long long` (`Int` is always 64-bit), and the `dp - empty` mental arithmetic trick.

## Everyday C++17 habits → CP Lang

| C++17 habit | Idiomatic CP Lang | Why |
| --- | --- | --- |
| `ios::sync_with_stdio(false); cin.tie(nullptr);` | Automatic | Fast buffered I/O is always on; `flush()` flushes interactive prompts. |
| `int T; cin >> T; while (T--) solve();` | `@!multitest` | Precomputation above `@!multitest` runs once; per-test state and `@memo` caches reset automatically. |
| `int n; cin >> n; vector<long long> a(n + 1);` | `let n: 1..=2e5 = read()` / `domain V = 1..=n` / `let a: [V]Int = read()` | Bounds are declared once and eliminate off-by-one and cross-index bugs. |
| `const long long INF = 1e18; dist[v] = min(dist[v], d);` | `var dist: [V]?Int = none` / `dist[v] min= d` | Niche-packed `?Int` uses `INT64_MIN` (0 extra bytes) and prevents `INF + w` overflow. |
| `((a % m) + m) % m` | `a % m` | `%` is Euclidean (`0 <= a % m < m`) for all integers. |
| `dx[4] / dy[4]` + bounds check | `type Cell = (Row, Col)` + `for q in p.adj4:` | Grid neighbors are guaranteed in-bounds without manual `0 <= nx && nx < h`. |
| `sort(a.begin(), a.end(), cmp)` | `sort(mut a, by: x => (x.r, desc(x.l)))` | Key functions compose tuples and `desc(...)` without writing strict-weak-order structs. |
| `priority_queue<T, vector<T>, greater<T>>` | `MinHeap<T>` / `MaxHeap<T>` | Direction is explicit in the type name. |
| `multiset::erase(s.find(x))` | `bag.remove_one(x)` | Never accidentally deletes all duplicates of `x`. |
| `vector<pair<int,int>> adj[n]` | `Graph<V>.undirected(...)` or `var adj: [V]List<V> = []` | `Graph<V>` lowers to a flat CSR array; `[V]List<V>` is indexed strictly by `V`. |
| `for (auto [u, v] : edges) { g[u].push_back(v); g[v].push_back(u); }` | `for (a, b) in [(u, v), (v, u)]:` or `Graph.undirected(...)` | One edge fact in one place (lint `L0503` catches duplicated symmetric writes). |
| `bool ok = false; for (...) if (p) { ok = true; break; }` | `let ok = any(p(x) for x in xs)` | Express search intent directly (lint `L0501` catches flag-and-break loops). |

## Sentinels and relaxation without `INF`

In C++, `INF = 1e18` is a constant source of `INF + w` overflow and forgotten `if (dist[u] == INF) continue;` guards. In CP Lang, `?Int` with `min=` / `max=` handles absence natively:

```cp CP Lang
domain V = 1..=4
var dist: [V]?Int = none
dist[V(1)] = 0
for (u, v, w) in [(V(1), V(2), 5), (V(2), V(4), 3), (V(1), V(4), 10)]:
    if let d = dist[u]:
        dist[v] min= d + w
print(dist[V(4)] ?? -1)
```

## Named invariants instead of parallel arrays

When a C++ solution keeps three parallel vectors (`stk_len`, `stk_val`, `offset`) or a raw segment-tree node array in sync by hand, give the invariant a `struct` and a combining function:

```cp CP Lang
// Maximum subarray sum monoid: no fake sentinels, one combining rule.
struct Seg(total: Int, pref: Int, suff: Int, best: Int)

fn combine(a: Seg, b: Seg) -> Seg:
    Seg(
        total: a.total + b.total,
        pref: max(a.pref, a.total + b.pref),
        suff: max(b.suff, b.total + a.suff),
        best: max(max(a.best, b.best), a.suff + b.pref),
    )

let x = combine(Seg(3, 3, 3, 3), Seg(-1, 0, 0, 0))
let y = combine(x, Seg(4, 4, 4, 4))
print(y.best)
```

## What `cpl build` emits

`cpl build solve.cp -o solve.cpp` produces a single, self-contained, human-readable C++17 file for the target judge profile:

- **Original `.cp` source included as a header comment** at the top of the `.cpp` file so judges and reviewers always see the clean source.
- **Tree-shaken standard and local libraries:** only reachable data structures and helpers are emitted; `@brute`, `@checker`, `@interactor`, `@gen`, `@test`, and `@law` blocks are stripped so submissions stay well under 64 KB.
- **Zero-cost domains and indices:** `domain V = 1..=n` erases to a plain `int32_t` index; `[V]T` lowers to a contiguous flat `std::vector<T>` with zero bounds checks on valid `V` accesses.
- **Niche-packed optionals:** `?Int` (`INT64_MIN`), `?F64` (quiet `NaN`), `?V`, `?Bool`, `?Char`, and `?Enum` take zero extra bytes — no `std::optional` padding overhead.
- **Compact `Mod<P>`:** stored as a 4-byte `uint32_t` with compile-time modulus reduction when `P` is `const`.
- **Flat CSR graphs:** `Graph<V>` packs adjacency lists into two contiguous arrays (`head` + `edges`) rather than `vector<vector<...>>`, avoiding heap fragmentation and pointer chasing.
- **Salted hash maps:** `HashMap` uses splitmix64 with a runtime-seeded salt to resist predictable anti-hash tests.
- **Smart `@memo` lowering:** state spaces up to `2^24` entries lower to a flat contiguous table; larger spaces fall back automatically to a salted hash map.

## Judge profiles

`cpl build` targets one named judge profile: `generic` (default), `codeforces`, `atcoder`, `cses`, `icpc` (DOMjudge), `kattis`, or `dmoj`. Each profile fixes compiler flags, platform, stack size, and source-size limit. `cpl judges` lists all profiles and marks the active one.

Selection order (first match wins):

- `cpl build a.cp --judge atcoder`
- a `// judge: atcoder` comment in the file
- `judge = "atcoder"` under `[judge]` in the nearest `cp.toml`

Custom profiles in `cp.toml` can extend a built-in judge:

```toml
[judge]
judge = "school"

[judge.school]
extends = "icpc"
stack_mb = 64              # or "unlimited"
source_kb = 100
flags = ["-std=c++17", "-O2"]
int128 = false             # `I128` becomes a build error naming the judge
```

What the profile changes in the file: the header names the judge and its command; the program runs on a thread with the profile's thread stack (1 GB unless `thread_stack_mb` says otherwise; with `thread_stack_mb = 0`, for a judge that allows no threads, `main` runs it and `cpl build` warns); the source limit is the judge's, with a warning past three quarters of it and the source header left out (then the indentation halved) only when the file would not fit otherwise. `cpl build` prints the file's size as a share of that limit. `cpl run --release`, `cpl test` and `cpl stress` build the same C++ with the judge's flags (those the compiler here accepts) and run it on the judge's stack.

## Performance controls and escape hatches

Clean high-level CP Lang matches hand-written `g++ -O2` C++17 within a narrow margin, and provides explicit escape hatches whenever a tight time limit demands raw control:

- **Plain integer indexing:** use structural range arrays such as `[0..n]Int` when an algorithm needs numeric offsets.
- **`unchecked:` block:** opt out of local overflow and bounds checks inside an inner hot loop you have already proved safe:

```cp CP Lang
let n = 3
let a: [0..n]Int = 2
let b: [0..n]Int = 3
var acc = 0
unchecked:
    for i in 0..n:
        acc += a[i] * b[i]
print(acc)
```

- **`O(1)` cleared arrays (`std.marks.Marks<D>`):** in multi-query algorithms where each query touches a few indices, use `Marks<D>` (32-bit generation stamp) or a touched-list instead of `a.fill(false)`.
- **Explicit iterative DP:** when a tight 2D DP needs a rolling 1D buffer for cache locality or memory limits, write the iterative loop directly with `[0..=cap]Int` — `@memo` is optional.
- **Inspect the C++17 anytime:** click **C++17** in the [playground](playground.html) or run `cpl build solve.cp` locally to verify the exact loop and memory layout `g++ -O2` will compile.

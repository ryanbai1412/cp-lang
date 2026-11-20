If you already write competitive programming solutions in C++17, you already know every algorithm you need. This guide maps everyday C++17 habits straight to idiomatic CP Lang 0.2.2 and explains how CP Lang compiles down to fast, readable, judge-ready C++17.

## Everyday C++17 habits in CP Lang

### Fast I/O and `#define int long long`

In C++, every submission starts with `ios::sync_with_stdio(false); cin.tie(nullptr);` and type aliases for 64-bit integers. In CP Lang, `Int` is always 64-bit, I/O is fast by default, and `read()` parses directly into typed containers:

```cpp C++17
ios::sync_with_stdio(false);
cin.tie(nullptr);
int n;
cin >> n;
vector<long long> a(n + 1);
for (int i = 1; i <= n; i++) cin >> a[i];
```

```cp CP Lang 0.2.2
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
```

### Multi-test boilerplate

In C++, you wrap everything in `void solve()` and remember to clear global vectors and DP tables between cases. In CP Lang, place precomputation above `@!multitest` and per-case code below it — scopes and `@memo` caches reset automatically, and `return` exits just the current case.

### Sentinels (`INF = 1e18`, `memset(dp, -1, sizeof(dp))`)

Replace `const long long INF = 1e18;` and `-1` unvisited flags with `?Int`. Because `?Int` uses `LLONG_MIN` as its niche tag, a `[V]?Int` array has the exact same byte size as `vector<int64_t>` while eliminating `INF + w` overflow bugs:

```cpp C++17
const long long INF = 1e18;
vector<long long> dist(n + 1, INF);
dist[src] = 0;
if (dist[u] < INF) dist[v] = min(dist[v], dist[u] + w);
cout << (dist[t] == INF ? -1 : dist[t]) << '\n';
```

```cp CP Lang 0.2.2
var dist: [V]?Int = none
dist[src] = 0
dist[v] min= dist[u] + w
print(dist[t] ?? -1)
```

### Negative modulo and 2D grids

- C++ `((a % m) + m) % m` becomes simply `a % m` (Euclidean modulo everywhere).
- C++ `const int dx[4] = {-1, 1, 0, 0};` + bounds check `0 <= nx && nx < n` becomes `type Cell = (Row, Col)` and `for q in p.adj4:`.
- C++ `struct Cmp` or lambda comparators for `std::sort` and `priority_queue` become `sort(mut a, by: x => (x.r, desc(x.l)))` and `MinHeap<(Int, Desc<V>)>`.
- C++ `map[k]` silent insertion becomes `m[k] -> ?V`, `m[k] += 1`, and `(m[k] ??= []).push(x)`.
- C++ `multiset::erase(s.find(x))` footgun becomes `SortedBag.remove_one(x)`.

## What `cpl build` emits

`cpl build solve.cp -o solve.cpp` produces a single, self-contained, human-readable C++17 file designed to pass every online judge (Codeforces, AtCoder, DMOJ, Kattis, ICPC):

- **Original `.cp` source included as a header comment** at the top of the `.cpp` file so judges and reviewers always see the clean source rather than opaque codegen.
- **Tree-shaken standard and local libraries:** only the data structures and helpers reachable from your program are emitted; `@brute`, `@checker`, `@interactor`, `@gen`, `@test`, and `@law` blocks are stripped completely so submissions stay compact (well under 64 KB).
- **Zero-cost domains and indices:** `domain V = 1..=n` erases to a plain `int32_t` index; `[V]T` lowers to a contiguous flat `std::vector<T>` with zero bounds checks on valid `V` accesses.
- **Niche-packed optionals:** `?Int` (`INT64_MIN`), `?F64` (quiet `NaN`), `?V`, `?Bool`, `?Char`, and `?Enum` take zero extra bytes — no `std::optional` padding overhead.
- **Compact `Mod<P>`:** stored as a 4-byte `uint32_t` with compile-time modulus reduction when `P` is `const`.
- **Flat CSR graphs:** `Graph<V>` packs adjacency lists into two contiguous arrays (`head` + `edges`) rather than `vector<vector<...>>`, avoiding heap fragmentation and pointer chasing.
- **Salted hash maps:** `HashMap` uses splitmix64 with a runtime-seeded salt to resist predictable collision patterns. For unique ordered keys, use `Set<T>`.
- **Smart `@memo` lowering:** state spaces up to `2^24` entries lower to a flat contiguous table; larger spaces fall back automatically to a salted hash map.

## Performance controls and escape hatches

CP Lang is designed so clean high-level code matches hand-written `g++ -O2` C++17 within a narrow margin, while giving you explicit escape hatches whenever a tight time limit demands raw control:

- **`a.raw[i]`:** index any domain-indexed array with a raw 0-based `Int` offset when translating an intricate pointer formula (bounds-checked in local `cpl run`, raw offset in release C++17).
- **`unchecked:` block:** opt out of local overflow and bounds checks inside an inner hot loop you have already proved safe:

```cp CP Lang 0.2.2
unchecked:
    for i in 0..n:
        acc += a.raw[i] * b.raw[i]
```

- **Touched-list reset instead of full `fill`:** in multi-query algorithms where each query only touches a few indices of a `[D]Int` or `[D]Bool` table, clear only the touched list (or import `std.marks.Marks<D>` for `O(1)` 32-bit generation-stamp resets).
- **Explicit iterative DP:** when a tight 2D DP needs a rolling 1D buffer for cache locality or memory limits, write the iterative loop directly with `[0..=cap]Int` — `@memo` is optional, never mandatory.
- **Inspect the C++17 anytime:** click **C++17** in the [playground](playground.html) or run `cpl build solve.cp` locally to verify the exact loop and memory layout `g++ -O2` will compile.

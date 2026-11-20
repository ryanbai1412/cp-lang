// Perf Repro #2 (P0 — #1 runtime outlier: dmopc21c8p5 34.82x, 164 -> 5699 ms;
//                also dmopc21c4p2 4.16x, ioi04p4 2.38x):
// `Map<Int, Int>` lowers to `std::map<ll, ll>` (red-black tree with per-node malloc
// and O(log N) pointer chasing), and `if cache.contains(x): return cache[x]` emits
// two full tree traversals (`cache.count(x)` followed by `cache.at(x)`).
// Fix: lower unordered `Map<K, V>` to a flat open-addressed hash table (with splitmix64
// anti-hash) and fold `contains` + `[]` / `get()` into a single lookup.

let n: Int = read()
var cache: Map<Int, Int> = Map()
var total = 0
for i in 1..=n:
    let k = (i * 2654435761) % 200003
    if cache.contains(k):
        total += cache[k]
    else:
        cache[k] = i
        total += i
print(total)

// Perf Repro #4 (P0 — #4 cco17p3 8.94x runtime & #2 RSS 9.99x = 2322 MB vs 232 MB):
// Fixed small domain array `[0..2]?Int` inside a `struct` emits heap-allocated
// `std::vector<std::optional<ll>>` (~80 B/node + per-node malloc) instead of
// inline `std::array<cpl::OptInt, 2>` or `int32_t[2]` (8–16 B/node, zero heap allocs).
// Fix: lower compile-time-constant small domain arrays (`[0..K]T` for small const K)
// inside structs to inline `std::array<T, K>` with niche-packed `cpl::OptInt`.

struct TrieNode:
    kid: [0..2]?Int
    val: Int

let n: Int = read()
var nodes: List<TrieNode> = []
for i in 0..n:
    var ch: [0..2]?Int = None
    nodes.push(TrieNode(ch, i))
print(nodes.len)

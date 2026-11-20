// AtCoder DP G Longest Path: the longest path in a DAG, in edges.
let (n, m): (2..=1e5, 1..=1e5) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()

var out: [V]List<V> = []
for (u, v) in edges: out[u].push(v)

@memo
fn longest(u: V) -> Int:
    max(longest(v) + 1 for v in out[u]) ?? 0
print(max(longest(v) for v in V))

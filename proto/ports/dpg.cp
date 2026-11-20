// cp: 0.1 1f59ac0+dirty
// AtCoder DP G Longest Path: the longest path in a DAG, in edges.
let n: 2..=1e5 = read()
let m: 1..=1e5 = read()
domain V = 1..=n
domain E = 0..m
fn forward() -> [E](u: V, v: V):
    [E] _ => (V(rand(1, n - 1)), V(rand(2, n)))
let edges: [E](u: V, v: V) = read(gen: forward) where all(e.u < e.v for e in edges)

var out: [V]List<V> = []
for (u, v) in edges: out[u].push(v)

@memo
fn longest(u: V) -> Int:
    max(longest(v) + 1 for v in out[u]) ?? 0
print(max(longest(v) for v in V))

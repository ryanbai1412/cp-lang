// MCCC '22 J5 (CP Lang 0.2.2): maximum independent set over `Subset<V>`.
// original: corpus/submissions/4223449.cpp
let (n, m): (Int, Int) = read()
domain V = 1..=n
var edges: List<(V, V)> = []
for _ in 0..m:
    let (u, v): (V, V) = read()
    edges.push((u, v))

var ans = 0
for mask in Subset<V>:
    if all(not (u in mask and v in mask) for (u, v) in edges):
        ans max= popcount(mask.idx)
print(ans)

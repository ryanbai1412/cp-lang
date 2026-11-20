// YAC 1 P2 — original: corpus/submissions/4376253.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let n: 1..=200_000 = read()
domain V = 1..=n
let color: [V]Char = read()
let es: [0..n - 1](V, V) = read()
let t = Rooted<V>(Graph.undirected(es), V.first)

// down[u, b, w]: downward paths from u with b black and w white nodes, both under 3
domain K = 0..3
var down: [V, K, K]Int = 0
var good = 0

for u in t.postorder:
    let (ub, uw) = if color[u] == 'B': (1, 0) else: (0, 1)
    // FRICTION 46.3: small counters used as indices need K(...) around every sum
    down[u, K(ub), K(uw)] = 1
    good += 1
    for v in t.children(u):
        good += sum(down[u, a, b] * down[v, c, d]
                    for a in K for b in K for c in K for d in K
                    if a + c < 3 and b + d < 3)
        for a in K for b in K if a + ub < 3 and b + uw < 3:
            down[u, K(a + ub), K(b + uw)] += down[v, a, b]

print(n * (n + 1) / 2 - good)

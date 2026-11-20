// AAC 4 P5 — original: corpus/submissions/4140767.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, m, x, q): (1..=500_000, 0..=500_000, 1..=n, 0..=500_000) = read()
domain V = 1..=n
let req: [0..q](V, V) = read()

fn build() -> ?List<(V, V)>:
    var dsu = Dsu<V>()
    for (u, v) in req:
        dsu.unite(u, v)

    var pq = MaxHeap<(Int, V)>()
    for v in V if dsu.find(v) == v:
        pq.push((dsu.size(v), v))
    if pq.len < x:
        return none

    while pq.len > x:
        let (_, u) = pq.pop()!
        let (_, v) = pq.pop()!
        dsu.unite(u, v)
        let r = dsu.find(u)
        pq.push((dsu.size(r), r))

    // FRICTION 39.2: Dsu has no .groups() helper; manual root-to-bucket reindexing
    var comp_id: [V]?Int = none
    var comps: List<List<V>> = []
    while let (_, r) = pq.pop():
        comp_id[r] = comps.len
        comps.push([])
    for v in V:
        comps[comp_id[dsu.find(v)]!].push(v)

    var edges: List<(V, V)> = []
    for c in comps if c.len > 1:
        edges.push((c.last!, c[0]))
        for i in 0..c.len - 1:
            edges.push((c[i], c[i + 1]))
    if edges.len > m:
        return none

    'intra: for c in comps:
        for i in 0..c.len:
            for j in 0..c.len if i != j and (i + 1) % c.len != j:
                if edges.len == m: break 'intra
                edges.push((c[i], c[j]))

    'cross: for i in 0..comps.len:
        for j in i + 1..comps.len:
            for u in comps[i]:
                for v in comps[j]:
                    if edges.len == m: break 'cross
                    edges.push((u, v))

    if edges.len != m: none else: edges

if let edges = build():
    for (u, v) in edges:
        print(u, v)
else:
    print(-1)

# Split the Attractions with a minimal stdlib

Kept: the core language (domains, `[D]T`, `List`, `?T`, postfix `?`, `??`, `for … where`, `while`), plus
`map`, `rev`, `zip`, `sorted(by:)`, `first(where:)`. Dropped: `Graph`, `bfs_tree`, `centroid`,
`components`, `Partition`, `bfs_order`, `mark`, `mask`, `connected_take`.

```cp
domain Group = labels(1..=3)

@grader
fn find_split(n: I32, a: I32, b: I32, c: I32, p: List<I32>, q: List<I32>) -> List<I32>:
    domain V = n
    let N = V.count
    var adj: [V]List<V> = []
    for (x, y) in zip(p, q):
        adj[V.at(x)].push(V.at(y))
        adj[V.at(y)].push(V.at(x))

    let want: [Group]Int = [a, b, c]
    let by_size = Group.list().sorted(by: g => want[g])
    let (small, mid, big) = (by_size[0], by_size[1], by_size[2])
    let (s, m) = (want[small], want[mid])

    // spanning tree: BFS order + parents; subtree sizes bottom-up
    var parent: [V]?V = none
    var seen: [V]Bool = false
    var order = [V.first]
    seen[V.first] = true
    var i = 0
    while i < order.len:
        for w in adj[order[i]] if not seen[w]:
            seen[w] = true
            parent[w] = order[i]
            order.push(w)
        i += 1
    var size: [V]Int = 1, heavy: [V]Int = 0
    for v in order.rev():
        if let u = parent[v]:
            size[u] += size[v]
            heavy[u] max= size[v]
    let centre = V.first(where: v => max(heavy[v], N - size[v]) <= N / 2)!

    // pieces of the tree without the centre, top-down
    var id: [V]?Int = none
    var count = 0
    for v in order if v != centre:
        if let u = parent[v], u != centre:
            id[v] = id[u]
        else:
            id[v] = count
            count += 1
    domain Piece = count
    let of: [V]?Piece = id.map(x => Piece.at(x?))
    var psize: [Piece]Int = 0
    for v in V:
        if let x = of[v]: psize[x] += 1

    fn join() -> ?List<Piece>:
        var padj: [Piece]List<Piece> = []
        for (x, y) in zip(p, q):
            if let u = of[V.at(x)], w = of[V.at(y)], u != w:
                padj[u].push(w)
                padj[w].push(u)
        var seen: [Piece]Bool = false
        for start in Piece if not seen[start]:
            seen[start] = true
            var got = [start], total = psize[start], j = 0
            while j < got.len:
                for y in padj[got[j]] if not seen[y]:
                    seen[y] = true
                    got.push(y)
                    total += psize[y]
                    if total >= s: return got
                j += 1
        return none

    fn take(k: Int, inside: [V]Bool) -> List<V>:     // traps if inside has no connected k-set from start
        var seen: [V]Bool = false
        let start = V.first(where: v => inside[v])!
        seen[start] = true
        var got = [start], j = 0
        while got.len < k:
            for w in adj[got[j]] if inside[w] and not seen[w] and got.len < k:
                seen[w] = true
                got.push(w)
            j += 1
        return got

    let chosen = Piece.first(where: x => psize[x] >= s).map(x => [x]) ?? join() ?? return List.filled(n, 0)
    var picked: [Piece]Bool = false
    for x in chosen: picked[x] = true
    let region = V.map(v => picked[of[v]?] ?? false)

    var label: [V]Group = big
    for v in take(s, region): label[v] = small
    for v in take(m, V.map(v => not region[v])): label[v] = mid
    return label.map(x => I32(x.label)).list()
```

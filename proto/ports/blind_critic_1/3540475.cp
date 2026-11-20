// Corpus submission 3540475: 1D coordinate MST via Boruvka 2-tracker linear sweeps + builtin Dsu
let n: 1..=2e5 = read()
let m: 0..=2e5 = read()

domain V = 1..=n
domain E = 1..=m

let arr: [V]Int = read()
let free_edges: [E](u: V, v: V) = read()

var dsu = Dsu<V>()
for e in E:
    let ed = free_edges[e]
    dsu.unite(ed.u, ed.v)

var verts: List<V> = []
for v in V:
    verts.push(v)
let order = verts.sorted_by(v => arr[v])

var ans = 0
while dsu.count > 1:
    var best_w: [V]?Int = none
    var best_to: [V]?V = none

    // Left-to-right sweep: keep last two vertices from distinct components
    var p1: ?V = none
    var p2: ?V = none
    for v in order:
        let rv = dsu.find(v)
        var cand: ?V = none
        if let u1 = p1:
            if dsu.find(u1) != rv:
                cand = u1
            else:
                cand = p2
        if let u = cand:
            let w = arr[v] - arr[u]
            if best_w[rv] == none or w < best_w[rv]!:
                best_w[rv] = w
                best_to[rv] = u
        if p1 == none:
            p1 = v
        else:
            if dsu.find(p1!) == rv:
                p1 = v
            else:
                p2 = p1
                p1 = v

    // Right-to-left sweep
    var q1: ?V = none
    var q2: ?V = none
    for idx in (0..order.len).rev:
        let v = order[idx]
        let rv = dsu.find(v)
        var cand: ?V = none
        if let u1 = q1:
            if dsu.find(u1) != rv:
                cand = u1
            else:
                cand = q2
        if let u = cand:
            let w = arr[u] - arr[v]
            if best_w[rv] == none or w < best_w[rv]!:
                best_w[rv] = w
                best_to[rv] = u
        if q1 == none:
            q1 = v
        else:
            if dsu.find(q1!) == rv:
                q1 = v
            else:
                q2 = q1
                q1 = v

    for v in V:
        if dsu.find(v) == v:
            if let to = best_to[v]:
                if dsu.unite(v, to):
                    ans += best_w[v]!

print(ans)

// EGOI '23 P4 (Bikes vs. Cars): reconstruct a weighted graph whose all-pairs
// max-min and min-max bottleneck paths match two given symmetric matrices.
// original: corpus/submissions/5678052.cpp
let n: 1..=500 = read()
let k: 1..=1e9 = read()
domain V = 0..n

fn read_sym() -> [V, V]Int:
    var mat: [V, V]Int = 0
    for i in V:
        for j in ..i:
            let x: Int = read()
            mat[i, j] = x
            mat[j, i] = x
    mat

var arr_mx = read_sym()
var arr_mn = read_sym()
for i in V:
    for j in V: arr_mx[i, j] = k - arr_mx[i, j]
for i in V:
    arr_mn[i, i] = 1e9
    arr_mx[i, i] = -1

fn closure_mn(mut g: [V, V]Int) -> [V, V]Int:
    for i in V: g[i, i] = 1e9
    for mid in V:
        for u in V:
            for v in V: g[u, v] max= min(g[u, mid], g[mid, v])
    g

fn closure_mx(mut g: [V, V]Int) -> [V, V]Int:
    for i in V: g[i, i] = -1
    for mid in V:
        for u in V:
            for v in V: g[u, v] min= max(g[u, mid], g[mid, v])
    g

fn kruskal(edges: List<(V, V, Int)>) -> List<(V, V, Int)>:
    var dsu = Dsu<V>()
    var out: List<(V, V, Int)> = []
    for (u, v, w) in edges:
        if dsu.unite(u, v): out.push((u, v, w))
    out

fn solve():
    // FRICTION 38.10: structural value equality `==` / `!=` on 2D `[V, V]Int`
    // and passing `[V, V]Int` by value to `closure_mn` vs copy-on-call.
    if closure_mn(arr_mn) != arr_mn or closure_mx(arr_mx) != arr_mx:
        print("NO")
        return

    var edges: List<(V, V, Int)> = []
    var cg_mn: [V, V]Int = -1
    var cg_mx: [V, V]Int = 1e9
    for i in V:
        for j in ..i:
            if arr_mn[i, j] >= arr_mx[i, j]:
                for w in [arr_mn[i, j], arr_mx[i, j]]:
                    edges.push((i, j, w))
                    cg_mn[i, j] max= w
                    cg_mn[j, i] max= w
                    cg_mx[i, j] min= w
                    cg_mx[j, i] min= w

    if closure_mn(cg_mn) != arr_mn or closure_mx(cg_mx) != arr_mx:
        print("NO")
        return

    let asc = edges.sorted_by(e => e.2)
    let keep = kruskal(asc) + kruskal(asc.rev)
    print(keep.len)
    for (u, v, w) in keep: print(u, v, w)

solve()

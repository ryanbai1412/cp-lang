@!multitest
let n: Int = read()
domain V = 1..=n
domain E = 0..n - 1
let raw: [E](u: V, v: V, w: Int) = read()
let tree = Rooted(Graph.undirected([E] e => (u: raw[e].u, v: raw[e].v)), root: V(1))

var par_w: [V]Int = 0
for e in E:
    let (u, v, w) = raw[e]
    if tree.parent(u) == v: par_w[u] = w
    else: par_w[v] = w

let q: Int = read()
for _ in 0..q:
    let t: Int = read()
    var ans = 0
    var up: [V]Int = 0
    for v in tree.postorder:
        var cur_sum = 0
        for c in tree.children(v):
            cur_sum += min(t - up[c], par_w[c])
            cur_sum min= t
        let e_top = min(t - cur_sum, par_w[v])
        ans += t - cur_sum - e_top
        up[v] = e_top
    print(ans)

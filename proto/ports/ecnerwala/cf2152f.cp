struct Jump<D>:
    depth: [D]Int
    par: [D]D
    jloc: [D]D
    jdist: [D]Int

    fn new() -> Jump<D> = Jump([D] _ => 0, [D] v => v, [D] v => v, [D] _ => 0)

    fn link(mut self, i: D, p: D, w: Int):
        depth[i] = depth[p] + w
        par[i] = p
        if p != D.last and jloc[p] != D.last and self.jdist[p] == self.jdist[jloc[p]]:
            jdist[i] = 1 + self.jdist[p] * 2
            jloc[i] = jloc[jloc[p]]
        else:
            jdist[i] = 1
            jloc[i] = p

    fn lca(self, a0: D, b0: D) -> D:
        var a = a0
        var b = b0
        if depth[a] > depth[b]: a = par[a]
        if a == b: return a
        while par[a] != par[b]:
            if jloc[a] != jloc[b]:
                a = jloc[a]
                b = jloc[b]
            else:
                a = par[a]
                b = par[b]
        par[a]

@!multitest
let (n, z): (Int, Int) = read()
let x: [0..n]Int = read()
let q: Int = read()
let qs: [0..q](l: Int, r: Int) = read()

domain V = 0..=n
var j1 = Jump<V>()
var j = n
for i in (0..n).rev:
    while x[j - 1] - x[i] > z: j -= 1
    j1.link(V(i), V(j), 1)

var j2 = Jump<V>()
for i in (0..n).rev:
    let vi = V(i)
    let vn = V(i + 1)
    let lca = j1.lca(vi, vn)
    j2.link(vi, lca, j1.depth[vi] + j1.depth[vn] - 2 * j1.depth[lca])

for (l_in, r_in) in qs:
    let l0 = V(l_in - 1)
    let r = V(r_in)
    var cur = l0
    while j2.par[cur] < r:
        cur = if j2.jloc[cur] < r: j2.jloc[cur] else: j2.par[cur]
    var ans = j2.depth[l0] - j2.depth[cur]
    for st in [cur.ord, cur.ord + 1] if st < r_in:
        var c = V(st)
        while c < r:
            c = if j1.jloc[c] < r: j1.jloc[c] else: j1.par[c]
        ans += j1.depth[V(st)] - j1.depth[c]
    print(ans)

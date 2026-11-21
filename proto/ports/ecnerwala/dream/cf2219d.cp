// Codeforces 2219 D: MEX Replacement on Tree.
// Weight x is missing from root-to-u path iff u lies outside sub(pos[x]), so
// sweeping x = 0..n over preorder complements with a 2-slot DualSegTree gives both
// path MEX f[u] and second-missing g[u]. Replacing p[v] with f[v] only helps when
// p[v] > f[v]: in sub(v), vertices with f[u] > p[v] drop to p[v] (loss, swept in
// descending x) while vertices with f[u] == f[v] rise to min(g[u], p[v]) (gain, swept per f-bucket).

@!multitest
let n: 1..=2e5 = read()
domain V = 1..=n
let p: [V]0..n = read()
let edges: List<(V, V)> = read(n - 1)
let t = Rooted<V>(edges, root: V.first)

var pos: [0..n]V = V.first
for v in V: pos[p[v]] = v

struct TwoMin(a: ?Int = none, b: ?Int = none):
    fn push(self, x: Int) -> TwoMin =
        if self.a is none: TwoMin(x, none)
        elif self.b is none: TwoMin(self.a, x)
        else: self

    fn then(self, other: TwoMin) -> TwoMin:
        var out = self
        if let x = other.a: out = out.push(x)
        if let y = other.b: out = out.push(y)
        out

var missing = DualSegTree<t.Pre, TwoMin>(compose: (a, b) => a.then(b), fill: TwoMin())
for x in 0..n:
    let sub = t.sub(pos[x])
    let tag = TwoMin(x)
    missing.apply(t.Pre.bounds.first..sub.start, tag)
    missing.apply(sub.end..t.Pre.bounds.last, tag)

let f: [V]Int = [V] v => missing.get(t.pos[v]).a ?? n
let g: [V]Int = [V] v => missing.get(t.pos[v]).b ?? n

struct Stats(cnt: Int = 0, sum: Int = 0)
impl Add for Stats:
    fn add(a: Stats, b: Stats) -> Stats = Stats(a.cnt + b.cnt, a.sum + b.sum)
impl Sub for Stats:
    fn sub(a: Stats, b: Stats) -> Stats = Stats(a.cnt - b.cnt, a.sum - b.sum)

var by_f: [0..=n]List<V> = []
for v in V: by_f[f[v]].push(v)

var fw = Fenwick<t.Pre, Stats>()
var loss: [V]Int = 0
for x in (0..n).rev:
    for u in by_f[x + 1]:
        fw.add(t.pos[u], Stats(1, f[u]))
    let v = pos[x]
    let s = fw.sum(t.sub(v))
    loss[v] = s.sum - x * s.cnt

var gain: [V]Int = 0
var active_cnt = Fenwick<t.Pre, Int>()
var small_g = Fenwick<t.Pre, Stats>()
for k in 0..=n if not by_f[k].empty:
    let verts = by_f[k].sorted(by: u => g[u])
    let queries = by_f[k].filter(v => p[v] > k).sorted(by: v => p[v])
    for u in verts: active_cnt.add(t.pos[u], 1)
    var j = 0
    for v in queries:
        while j < verts.len and g[verts[j]] < p[v]:
            let u = verts[j]
            small_g.add(t.pos[u], Stats(1, g[u]))
            j += 1
        let total = active_cnt.sum(t.sub(v))
        let sm = small_g.sum(t.sub(v))
        gain[v] = (sm.sum - sm.cnt * k) + (total - sm.cnt) * (p[v] - k)
    for u in verts: active_cnt.add(t.pos[u], -1)
    for i in 0..j: small_g.add(t.pos[verts[i]], Stats(-1, -g[verts[i]]))

let base = f.sum()
let best_delta = V.map(v => if p[v] > f[v]: gain[v] - loss[v] else: 0).max() ?? 0
print(base + max(0, best_delta))

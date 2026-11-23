// cf1915g Bicycles: Dijkstra over (city, slowest-so-far bike) states. Riding edge w
// with bike b costs w * s[b]; arriving at u, keep whichever bike is faster.
@!multitest
let (n, m): (2..=1000, 1..=1000) = read()
domain V = 1..=n
let roads = read<[m](u: V, v: V, w: 1..=1e5)>()
let s = read<[V]1..=1000>()

var adj: [V]List<(to: V, w: Int)> = []
for r in roads:
    adj[r.u].push((to: r.v, w: r.w))
    adj[r.v].push((to: r.u, w: r.w))

domain S = (V, V)
fn better(b: V, u: V) -> V = if s[u] < s[b]: u else: b
let p = dijkstra((v, b) => adj[v].map(a => (to: (a.to, better(b, a.to)), w: a.w * s[b])),
    from: [((V.first!, V.first!), 0)])
print(V.map(b => p[(V.last!, b)]).present().min()!)

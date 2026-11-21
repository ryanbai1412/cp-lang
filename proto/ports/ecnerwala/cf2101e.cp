// For one distance budget: the deepest vertex with that budget, the child
// subtree it sits in, and the deepest one in any other subtree.
type Far<V> = (depth: Int, child: V)
struct Best<V>:
    far: ?Far<V>
    other: ?Int

fn deeper(a: ?Int, b: ?Int) -> ?Int:
    if let x = a, let y = b: return max(x, y)
    a ?? b

fn merge<V>(a: Best<V>, o: Best<V>) -> Best<V>:
    let x = a.far ?? return o
    let y = o.far ?? return a
    if x.child == y.child: Best((max(x.depth, y.depth), x.child), deeper(a.other, o.other))
    elif y.depth > x.depth: Best(y, deeper(x.depth, o.other))
    else: Best(x, deeper(a.other, y.depth))

@!multitest
let n: Int = read()
domain V = 1..=n
let s: [V]Char = read()
let edges: [0..n - 1](u: V, v: V) = read()
let adj = Graph.undirected(edges)

var ans: [V]Int = [V] v => if s[v] == '1': 1 else: -1
var min_dist: [V]Int = [V] v => if s[v] == '1': n else: 0
for round in 2..:
    var next_dist: [V]Int = 0
    var alive: [V]Bool = true
    var sz: [V]Int = 0
    var bests: List<Best<V>> = []

    fn calc_sz(cur: V, prv: ?V) -> Int:
        sz[cur] = 1 + sum(calc_sz(nxt, cur) for nxt in adj[cur] if alive[nxt] and nxt != prv)
        sz[cur]

    fn offer(cur: V, ch: V, depth: Int):
        if min_dist[cur] > 0 and min_dist[cur] >= depth:
            let idx = min(bests.len - 1, min_dist[cur] - depth)
            bests[idx] = merge(bests[idx], Best((depth, ch), none))

    fn collect(cur: V, prv: V, depth: Int, ch: V):
        offer(cur, ch, depth)
        for nxt in adj[cur] if alive[nxt] and nxt != prv:
            collect(nxt, cur, depth + 1, ch)

    fn query(cur: V, ch: V, depth: Int):
        let b = bests[depth]
        let top = b.far ?? return
        let d = if top.child == ch: b.other else: top.depth
        if let far = d, s[cur] == '1': next_dist[cur] max= depth + far

    fn answer(cur: V, prv: V, depth: Int, ch: V):
        query(cur, ch, depth)
        for nxt in adj[cur] if alive[nxt] and nxt != prv:
            answer(nxt, cur, depth + 1, ch)

    fn decompose(start: V):
        var rt = start
        let total = calc_sz(rt, none)
        while let nxt = adj[rt].first(w => alive[w] and sz[w] < sz[rt] and sz[w] * 2 >= total):
            rt = nxt
        bests = List.filled(total + 1, Best(none, none))
        offer(rt, rt, 0)
        for ch in adj[rt] if alive[ch]: collect(ch, rt, 1, ch)
        for i in (0..total).rev: bests[i] = merge(bests[i], bests[i + 1])
        query(rt, rt, 0)
        for ch in adj[rt] if alive[ch]: answer(ch, rt, 1, ch)
        alive[rt] = false
        for ch in adj[rt] if alive[ch]: decompose(ch)

    decompose(V.first)
    let reached = [v for v in V if next_dist[v] > 0]
    if reached.is_empty: break
    for v in reached: ans[v] = round
    min_dist = [V] v => next_dist[v] / 2
print(ans)

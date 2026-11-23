// Tuned: costflow.cp's successive shortest paths written out with its own
// binary heap. `CostFlow.slope` runs Dijkstra on `MinHeap`, whose every
// comparison first tests its optional `by:` comparator (perf PLAN.md).
domain V = 0..2002
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

let (s, t) = (2000, 2001)
var head: [V]List<Int> = []
var to: List<Int> = []
var left: List<Int> = []
var price: List<Int> = []
fn add(u: Int, v: Int, c: Int, w: Int):
    head[V(u)].push(to.len)
    to.push(v)
    left.push(c)
    price.push(w)
    head[V(v)].push(to.len)
    to.push(u)
    left.push(0)
    price.push(-w)

for i in 0..1000:
    add(s, i, 5, 0)
    add(1000 + i, t, 5, 0)
for _ in 0..60000:
    let u = next(1000)
    let v = next(1000)
    let c = next(5) + 1
    add(u, 1000 + v, c, next(10000))

var hd: List<Int> = []
var hv: List<Int> = []
fn push(d: Int, v: Int):
    var i = hd.len
    hd.push(d)
    hv.push(v)
    unchecked:
        while i > 0:
            let p = (i - 1) / 2
            if hd[p] <= d: break
            hd[i] = hd[p]
            hv[i] = hv[p]
            i = p
        hd[i] = d
        hv[i] = v

fn pop_top():
    let d = hd.pop()!
    let v = hv.pop()!
    let n = hd.len
    if n == 0: return
    var i = 0
    unchecked:
        while 2 * i + 1 < n:
            var c = 2 * i + 1
            if c + 1 < n and hd[c + 1] < hd[c]: c += 1
            if hd[c] >= d: break
            hd[i] = hd[c]
            hv[i] = hv[c]
            i = c
        hd[i] = d
        hv[i] = v

var pot: [V]Int = 0
var (flow, cost) = (0, 0)
unchecked:
    while true:
        var dist: [V]Int = Int.MAX
        var via: [V]Int = -1
        var done: [V]Bool = false
        dist[V(s)] = 0
        push(0, s)
        while hd.len > 0:
            let (d, u) = (hd[0], V(hv[0]))
            pop_top()
            if done[u]: continue
            done[u] = true
            if u.ord == t: break
            for a in head[u]:
                if left[a] == 0: continue
                let v = V(to[a])
                let nd = d + price[a] + pot[u] - pot[v]
                if nd < dist[v]:
                    dist[v] = nd
                    via[v] = a
                    push(nd, v.ord)
        hd.clear()
        hv.clear()
        if not done[V(t)]: break
        for v in V: pot[v] += if done[v]: dist[v] else: dist[V(t)]
        var amount = Int.MAX
        var v = t
        while v != s:
            amount min= left[via[V(v)]]
            v = to[via[V(v)] ^ 1]
        v = t
        while v != s:
            let a = via[V(v)]
            left[a] -= amount
            left[a ^ 1] += amount
            v = to[a ^ 1]
        flow += amount
        cost += amount * (pot[V(t)] - pot[V(s)])
print(flow, cost)

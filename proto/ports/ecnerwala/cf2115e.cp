let (n, m): (Int, Int) = read()
domain V = 1..=n
type Item = (cost: Int, worth: Int)
let item: [V]Item = read()
let edges: [0..m](u: V, v: V) = read()
let next = Graph.directed(edges)
let q: Int = read()
type Query = (at: V, budget: Int)
let queries: [0..q]Query = read()

let cap = max(item[v].cost for v in V)! ** 2
var small: [V, 0..=cap]Int = 0
for v in V:
    let (c, w) = item[v]
    for j in 0..=cap - c:
        small[v, j + c] max= small[v, j] + w
    for k in next[v]:
        for j in 0..=cap:
            small[k, j] max= small[v, j]
var ans = [small[x.at, min(x.budget, cap)] for x in queries]

for pivot in V:
    let (pc, pw) = item[pivot]
    var reach: [V]Bool = false
    var best: [V, 0..pc]Int = 0
    reach[V.first] = true
    for v in V if reach[v]:
        let (c, w) = item[v]
        if w * pc <= pw * c:
            let g = gcd(c, pc)
            for start in 0..g:
                for step in 0..=2 * (pc / g):
                    let j = (start + step * c) % pc
                    best[v, (j + c) % pc] max= best[v, j] + w - (j + c) / pc * pw
        for k in next[v]:
            if v < pivot and pivot < k: continue
            reach[k] = true
            for j in 0..pc:
                best[k, j] max= best[v, j]
    for (i, x) in queries.items:
        if x.at >= pivot and reach[x.at] and x.budget >= cap:
            ans[i] max= best[x.at, x.budget % pc] + x.budget / pc * pw

print(ans, sep: "\n")

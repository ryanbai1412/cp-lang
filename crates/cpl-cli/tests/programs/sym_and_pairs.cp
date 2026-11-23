domain V = 1..=read<Int>()
var adj: [Sym<V>]Bool = false
var wt: [Sym<V>]Int = 0
let m = read<Int>()
repeat m:
    let (u, v): (V, V) = read()
    adj[u, v] = true
    wt[u, v] += 10
    wt[v, u] += 5

var edges = 0
var weight_sum = 0
for (i, j) in V.pairs:
    if not adj[j, i]: continue
    edges += 1
    let p = (i, j)
    weight_sum += wt[p]

var after_cnt = 0
var before_cnt = 0
for i in V:
    for j in V.after(i):
        if adj[i, j]: after_cnt += 1
    for j in V.before(i):
        if adj[j, i]: before_cnt += 1

var first_hit = 0
for (i, j) in V.pairs:
    if adj[i, j]:
        first_hit = i.label * 10 + j.label
        break

let all_pairs = V.pairs
domain P = V.pairs
print(edges, weight_sum, after_cnt, before_cnt, first_hit, all_pairs.len, P.count)

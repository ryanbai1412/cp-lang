// YAC 1 P2 (today/ runnable subset) — original: corpus/submissions/4376253.cpp

let n: 1..=200_000 = read()
let s: [0..n]Char = read()
var adj: [0..n]List<Int> = []
for i in 0..n - 1:
    let u: 1..=200_000 = read()
    let v: 1..=200_000 = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var par: [0..n]Int = -1
var order: List<Int> = []
order.push(0)
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for j in 0..adj[u].len:
        let v = adj[u][j]
        if v != par[u]:
            par[v] = u
            order.push(v)

// vals[u, a, b]: downward paths from u with a 'B' and b other nodes, both below 3
var vals: [0..n, 0..3, 0..3]Int = 0
var good = 0
for t in 0..n:
    let u = order[n - 1 - t]
    var ax = 0
    var bx = 1
    if s[u] == 'B':
        ax = 1
        bx = 0
    good += 1
    vals[u, ax, bx] = 1
    for j in 0..adj[u].len:
        let v = adj[u][j]
        if v != par[u]:
            for a in 0..3:
                for b in 0..3:
                    for c in 0..3 - a:
                        for d in 0..3 - b:
                            good += vals[u, a, b] * vals[v, c, d]
            for a in 0..3 - ax:
                for b in 0..3 - bx:
                    vals[u, a + ax, b + bx] += vals[v, a, b]

print(n + n * (n - 1) / 2 - good)

// Tuned: matching.cp's Hopcroft-Karp written out as matching.cpp writes it, in
// 32-bit arrays with -1 for "none", instead of std's `max_matching`.
const N = 100_000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var adj: [0..N]List<0..N> = []
for _ in 0..400_000:
    let u = next(N)
    adj[u].push(next(N))

var mate: [0..N]-1..N = -1
var back: [0..N]-1..N = -1
var level: [0..N]-1..N = -1
var nx: [0..N]0..=400_000 = 0

fn layers():
    var q: List<0..N> = []
    for u in 0..N:
        level[u] = -1
        if mate[u] < 0:
            level[u] = 0
            q.push(u)
    var at = 0
    while at < q.len:
        let u = q[at]
        at += 1
        for v in adj[u]:
            let w = back[v]
            if w >= 0 and level[w] < 0:
                level[w] = level[u] + 1
                q.push(w)

var size = 0
for u in 0..N:
    for v in adj[u]:
        if back[v] < 0:
            mate[u] = v
            back[v] = u
            size += 1
            break
while true:
    layers()
    var any = false
    for u in 0..N:
        if any: break
        if level[u] >= 0:
            for v in adj[u]:
                if back[v] < 0:
                    any = true
                    break
    if not any: break
    for u in 0..N: nx[u] = 0
    for root in 0..N:
        if mate[root] >= 0: continue
        var path: List<0..N> = [root]
        while not path.is_empty:
            let u = path.last!
            if nx[u] == adj[u].len:
                level[u] = -1
                let _ = path.pop()
                continue
            let v = adj[u][nx[u]]
            nx[u] += 1
            let w = back[v]
            if w >= 0:
                if level[w] == level[u] + 1: path.push(w)
            else:
                var fr: Int = v
                for i in (0..path.len).rev:
                    let x = path[i]
                    let old = mate[x]
                    mate[x] = fr
                    back[fr] = x
                    if old >= 0: fr = old
                size += 1
                path = []
print(size)

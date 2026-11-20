@!multitest
let (n, m, k): (Int, Int, Int) = read()
var adj: [0..n]List<Int> = []
var out_deg: [0..n]Int = [0..n] i => (n - 1 - i) % 2
let tgt_1s = n / 2
for _ in 0..m:
    let (u0, v0): (Int, Int) = read()
    let u = u0 - 1
    let v = v0 - 1
    adj[u].push(v)
    adj[v].push(u)
    if u > v:
        out_deg[u] ^= 1
        out_deg[v] ^= 1

if k % 2 == 0:
    print("Yes")
else:
    for i in 0..n: adj[i].sort()
    var unvisited: Map<Int, Bool> = Map()
    for i in 0..n: unvisited[i] = true
    var mi_1 = 0
    var ma_1 = 0
    for start in 0..n:
        if start not in unvisited: continue
        var c0 = 0
        var c1 = 0
        fn dfs(cur: Int):
            unvisited.remove(cur)
            if out_deg[cur] == 0: c0 += 1 else: c1 += 1
            var nxt = 0
            var ai = 0
            while let entry = unvisited.first_ge(nxt):
                nxt = entry.0
                while ai < adj[cur].len and adj[cur][ai] < nxt: ai += 1
                if not (ai < adj[cur].len and adj[cur][ai] == nxt):
                    dfs(nxt)
                nxt += 1
        dfs(start)
        let tot = c0 + c1
        var mi = 0
        var ma = tot
        if mi % 2 != c1 % 2: mi += 1
        if ma % 2 != c1 % 2: ma -= 1
        mi_1 += mi
        ma_1 += ma
    print(if mi_1 <= tgt_1s and tgt_1s <= ma_1: "Yes" else: "No")

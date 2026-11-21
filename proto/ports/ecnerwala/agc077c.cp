@!multitest
let (n, m, k): (Int, Int, Int) = read()
var adj: [0..n]List<Int> = []
var out_deg = [0..n] i => (n - 1 - i) % 2
for _ in 0..m:
    let (u0, v0): (Int, Int) = read()
    let (u, v) = (u0 - 1, v0 - 1)
    adj[u].push(v)
    adj[v].push(u)
    if u > v:
        out_deg[u] ^= 1
        out_deg[v] ^= 1

if k % 2 == 0:
    print("Yes")
    return

for i in 0..n: adj[i].sort()
var unvisited = Set(0..n)

var mi_1 = 0
var ma_1 = 0
for start in 0..n if start in unvisited:
    var c0 = 0
    var c1 = 0
    fn dfs(cur: Int):
        unvisited.remove(cur)
        if out_deg[cur] == 0: c0 += 1 else: c1 += 1
        var nxt = 0
        var ai = 0
        while let v = unvisited.first_ge(nxt):
            nxt = v
            while ai < adj[cur].len and adj[cur][ai] < nxt: ai += 1
            if ai == adj[cur].len or adj[cur][ai] != nxt: dfs(nxt)
            nxt += 1
    dfs(start)
    let tot = c0 + c1
    mi_1 += c1 % 2
    ma_1 += tot - (tot - c1) % 2

let tgt = n / 2
print(if mi_1 <= tgt and tgt <= ma_1: "Yes" else: "No")

type Z = Mod<998244353>

@!multitest
let (n, m, v0): (Int, Int, Int) = read()
let v_mod: Z = v0
let a: List<Int> = [read::<Int>() for _ in 0..n]
var adj: [0..n]List<Int> = []
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var depth: [0..n]?Int = none
var low: [0..n]Int = 0
var must_val: [0..n]?Int = none
var ans: Z = 1

fn dfs(cur: Int, prv: Int):
    let d = if prv < 0: 0 else: depth[prv]! + 1
    depth[cur] = d
    low[cur] = d
    if a[cur] >= 0: must_val[cur] = a[cur]
    for nxt in adj[cur]:
        if nxt == prv: continue
        if let nd = depth[nxt]:
            if nd < d:
                low[cur] min= nd
                if ((1 ^ nd ^ d) & 1) != 0:
                    if let mv = must_val[cur]:
                        if mv != 0: ans = 0
                    else:
                        must_val[cur] = 0
        else:
            dfs(nxt, cur)
            if low[nxt] <= d and low[nxt] != depth[nxt]!:
                low[cur] min= low[nxt]
                if let nm = must_val[nxt]:
                    if let cm = must_val[cur]:
                        if cm != nm: ans = 0
                    else:
                        must_val[cur] = nm
    if low[cur] == d and must_val[cur] == none:
        ans *= v_mod

dfs(0, -1)
print(ans)

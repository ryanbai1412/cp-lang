@!multitest
let n: Int = read()
let s: Str = read()
var adj: [0..n]List<Int> = []
for _ in 0..(n - 1):
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var rt = 0
while s[rt] != '1': rt += 1

let inf = 1e99
fn dfs(cur: Int, prv: Int) -> (F64, F64):
    var base = 0.0
    var ch: List<F64> = []
    for nxt in adj[cur]:
        if nxt == prv: continue
        let v = dfs(nxt, cur)
        ch.push(v.1 - v.0)
        base += v.0
    if s[cur] == '1': return (base, base)
    ch.sort()
    var b0 = inf
    var b1 = inf
    let deg = F64(adj[cur].len)
    for cnt in 0..=ch.len:
        if cnt > 0:
            base += ch[cnt - 1]
            b1 min= base + deg / F64(cnt)
        b0 min= base + deg / F64(cnt + 1)
    (b0, b1)

let ans = dfs(rt, -1)
print(ans.0)

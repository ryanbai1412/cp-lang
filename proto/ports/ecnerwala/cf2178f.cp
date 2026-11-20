const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
var adj: [0..n]List<Int> = []
for _ in 0..(n - 1):
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var comps = 0
var ans: Z = 1
var s_inv: Z = 0
let one: Z = 1

fn dfs(cur: Int, prv: Int) -> Int:
    var sz = 1
    for nxt in adj[cur]:
        if nxt != prv:
            sz += dfs(nxt, cur)
    if prv < 0:
        ans *= sz
    elif sz % 2 == 0:
        let zsz: Z = sz
        ans *= zsz
        comps += 1
        ans *= comps
        ans *= zsz
        s_inv += one / zsz
        sz = 0
    sz

let _ = dfs(0, -1)
if comps > 0:
    let zc: Z = comps
    ans *= s_inv
    ans /= zc
else:
    ans = 1
print(ans)

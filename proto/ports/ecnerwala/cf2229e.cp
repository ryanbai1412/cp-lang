// Codeforces 2229 E: root the tree at N - 1 to record each vertex's highest
// child-subtree maximum, then run a prefix-sum modular DP from max_leaf to N - 1.
const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
var adj: List<List<Int>> = List.filled(n, [])
for _ in 0..n - 1:
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var max_leaf = 0
var prv_val: [0..n]?Int = none

fn dfs(cur: Int, prv: ?Int) -> Int:
    var max_ch: ?Int = none
    if adj[cur].len == 1: max_leaf max= cur
    for nxt in adj[cur]:
        if some(nxt) == prv: continue
        let ch = dfs(nxt, cur)
        if cur == n - 1 and ch == n - 2: continue
        if let prev = max_ch: max_ch = max(prev, ch)
        else: max_ch = ch
    prv_val[cur] = max_ch
    if let mc = max_ch: max(mc, cur) else: cur

let _ = dfs(n - 1, none)
var dp: [0..n]Z = 0
var dp_pref: [0..=n]Z = 0
dp[max_leaf] = 1
for i in max_leaf + 1..n:
    dp_pref[i] = dp_pref[i - 1] + dp[i - 1]
    if let pv = prv_val[i]:
        if pv < i: dp[i] = dp_pref[i] - dp_pref[pv + 1]
print(dp[n - 1])

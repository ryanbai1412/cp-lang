// Codeforces 2229 I: rerooting tree DP using max-plus convolution bottom-up
// and transposed max-plus convolution top-down over prefix child products.
const INF = 1_000_000_000_000_000_000

fn conv_vecs(a: List<Int>, b: List<Int>) -> List<Int>:
    var res = List.filled(a.len + b.len - 1, -INF)
    for i in 0..a.len:
        for j in 0..b.len:
            res[i + j] max= a[i] + b[j]
    res

fn conv_transpose(a: List<Int>, b: List<Int>) -> List<Int>:
    var res = List.filled(a.len - b.len + 1, -INF)
    for i in 0..res.len:
        for j in 0..b.len:
            res[i] max= a[i + j] + b[j]
    res

@!multitest
let (n, k): (Int, Int) = read()
let w: [0..n]Int = read()
var raw_adj: List<List<Int>> = List.filled(n, [])
for _ in 0..n - 1:
    let (u, v): (Int, Int) = read()
    raw_adj[u - 1].push(v - 1)
    raw_adj[v - 1].push(u - 1)

var ch: List<List<Int>> = List.filled(n, [])
var sz: [0..n]Int = 1

fn build_tree(cur: Int, prv: ?Int):
    var kids: List<Int> = []
    for nxt in raw_adj[cur]:
        if some(nxt) == prv: continue
        build_tree(nxt, cur)
        sz[cur] += sz[nxt]
        kids.push(nxt)
    ch[cur] = sorted(kids, by: u => sz[u])

build_tree(0, none)

var dp_down: List<List<Int>> = List.filled(n, [])
var prefs: List<List<List<Int>>> = List.filled(n, [])

fn dfs_down(cur: Int):
    var cur_dp: List<Int> = [0]
    var cur_prefs: List<List<Int>> = []
    for nxt in ch[cur]:
        dfs_down(nxt)
        cur_prefs.push(cur_dp)
        cur_dp = conv_vecs(cur_dp, dp_down[nxt])
    cur_dp.push(-INF)
    for i in (1..cur_dp.len).rev:
        cur_dp[i] max= cur_dp[i - 1] + w[cur] * i
    dp_down[cur] = cur_dp
    prefs[cur] = cur_prefs

dfs_down(0)

var ans: [0..n]Int = 0
var dp_up: List<List<Int>> = List.filled(n, [])
var root_up = List.filled(n + 1, -INF)
root_up[k] = 0
dp_up[0] = root_up

fn dfs_up(cur: Int):
    var cur_up = dp_up[cur]
    for z in (0..ch[cur].len).rev:
        let nxt = ch[cur][z]
        var nxt_up = conv_transpose(cur_up, prefs[cur][z])
        for i in 0..nxt_up.len - 1:
            nxt_up[i] max= nxt_up[i + 1] + w[cur] * (k - i)
        nxt_up.pop()
        dp_up[nxt] = nxt_up
        cur_up = conv_transpose(cur_up, dp_down[nxt])
    ans[cur] = cur_up[1] + w[cur] * k
    for nxt in ch[cur]: dfs_up(nxt)

dfs_up(0)
print(ans)

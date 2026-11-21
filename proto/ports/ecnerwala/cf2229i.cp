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
    let alen = a.len
    let aa: [0..alen]Int = [0..alen] idx => a[idx]
    var res = List.filled(alen - b.len + 1, -INF)
    for i in 0..alen - b.len + 1:
        for j in 0..b.len:
            res[i] max= aa[i + j] + b[j]
    res

fn shift_down(mut dp: List<Int>, weight: Int):
    dp.push(-INF)
    for i in (1..dp.len).rev:
        dp[i] max= dp[i - 1] + weight * i

@! multitest
let (n, k): (Int, Int) = read()
domain V = 1..=n
let w: [V]Int = read()
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))
let ch: [V]List<V> = [V] u => sorted(t.children(u), by: v => t.size(v))

var dp_down: [V]List<Int> = []
var prefs: [V]List<List<Int>> = []
for cur in t.postorder:
    var cur_dp: List<Int> = [0]
    var cur_prefs: List<List<Int>> = []
    for nxt in ch[cur]:
        cur_prefs.push(cur_dp)
        cur_dp = conv_vecs(cur_dp, dp_down[nxt])
    shift_down(mut cur_dp, w[cur])
    dp_down[cur] = cur_dp
    prefs[cur] = cur_prefs

var ans: [V]Int = 0
var dp_up: [V]List<Int> = []
dp_up[V(1)] = [if i == k: 0 else: -INF for i in 0..=n]
for cur in t.preorder:
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

print(ans)

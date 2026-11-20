@!multitest
let n: Int = read()
var adj: [0..n]List<(Int, Int)> = []
for _ in 0..(n - 1):
    let (u, v, w): (Int, Int, Int) = read()
    adj[u - 1].push((v - 1, w))
    adj[v - 1].push((u - 1, w))

let q: Int = read()
for _ in 0..q:
    let t: Int = read()
    var ans = 0

    fn dfs(cur: Int, prv: Int, par_w: Int) -> Int:
        var cur_sum = 0
        for (nxt, w) in adj[cur]:
            if nxt == prv: continue
            let ch = min(t - dfs(nxt, cur, w), w)
            cur_sum = min(cur_sum + ch, t)
        let e_top = min(t - cur_sum, par_w)
        ans += t - cur_sum - e_top
        e_top

    let _ = dfs(0, -1, 0)
    print(ans)

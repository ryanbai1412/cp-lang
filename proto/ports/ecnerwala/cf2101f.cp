const MOD = 998244353

@! multitest
let orig_n: Int = read()
let tot = 2 * orig_n - 1
var adj: [0..tot]List<Int> = []
for e in 0..orig_n - 1:
    let (u, v): (Int, Int) = read()
    let mid = orig_n + e
    adj[u - 1].push(mid)
    adj[v - 1].push(mid)
    adj[mid].push(u - 1)
    adj[mid].push(v - 1)
var p2: [0..=orig_n]Mod<MOD> = 1
var p3: [0..=orig_n]Mod<MOD> = 1
for i in 1..=orig_n:
    p2[i] = p2[i - 1] * 2
    p3[i] = p3[i - 1] * 3
var ans: Mod<MOD> = 0
for rt in 0..tot:
    var ways_red: [0..tot]Mod<MOD> = 0
    var ways_rb: [0..tot]Mod<MOD> = 0
    var dist_tot: [0..tot]Int = 0
    var dist_cur: [0..tot]Int = 0
    if rt < orig_n:
        dist_tot[0] += 1

    fn dfs(cur: Int, prv: Int, depth: Int):
        if cur < orig_n:
            dist_tot[depth] += 1
            dist_cur[depth] += 1
        for nxt in adj[cur]:
            if nxt != prv:
                dfs(nxt, cur, depth + 1)

    let start_d = if rt < orig_n: 0 else: 1
    for ch in adj[rt]:
        dfs(ch, rt, 1)
        var d = start_d
        while d < tot:
            let c = dist_cur[d]
            ways_red[d] -= p2[c] - 1
            ways_rb[d] -= p3[c] - p2[c] * 2 + 1
            dist_cur[d] = 0
            d += 2
    var pref_rb: Mod<MOD> = 1
    var red_only: Mod<MOD> = 0
    var ans_red: Mod<MOD> = 0
    var d = start_d
    while d < tot:
        let c = dist_tot[d]
        ways_red[d] += p2[c] - 1
        ways_rb[d] += p3[c] - p2[c] * 2 + 1
        let md: Mod<MOD> = d
        ans += md * 2 * pref_rb * ways_rb[d]
        ans += (ans_red + md * red_only) * ways_red[d] * 2
        red_only *= p2[c]
        ans_red *= p2[c]
        let diff = p3[c] - p2[c]
        red_only += pref_rb * diff
        ans_red += md * pref_rb * diff
        pref_rb *= p3[c]
        d += 2
print(ans / 2)

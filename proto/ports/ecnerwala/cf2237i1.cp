// Codeforces 2237 I1: bottom-up tree DP counting canonical valid assignments
// by depth and (has_active, has_next) cut flags.
type Z = Mod<1_000_000_007>

@! multitest
let n: Int = read()
domain V = 1..=n
let s_in: Str = read()
let s = "0" + s_in

var ch: [V]List<V> = []
for u in V:
    let l: Int = read()
    let kids: [0..l]V = read()
    for v in kids: ch[u].push(v)

var dp: [V]List<[0..2, 0..2]Z> = []

fn dfs(cur: V):
    var base: [0..2, 0..2]Z = 0
    base[0, 0] = 1
    var cur_dp: List<[0..2, 0..2]Z> = [base]
    for nxt in ch[cur]:
        dfs(nxt)
        let nxt_dp = dp[nxt]
        let sz = max(cur_dp.len, nxt_dp.len + 1)
        var ndp: [0..sz, 0..2, 0..2]Z = 0
        let ch_s = s[nxt.ord]
        for d0 in 0..cur_dp.len:
            for d1 in 0..nxt_dp.len:
                for ca in 0..2:
                    for cn in 0..2:
                        let v_cur = cur_dp[d0][ca, cn]
                        if v_cur == 0: continue
                        for na in 0..2:
                            for nn in 0..2:
                                let v_nxt = nxt_dp[d1][na, nn]
                                if v_nxt == 0: continue
                                let v = v_cur * v_nxt
                                for e in 0..2:
                                    if ch_s != '?' and ch_s != Char('0'.ord + e): continue
                                    var (n_na, n_nn, nd1) = (na, nn, d1 + e)
                                    if e == 1 and na == 1:
                                        (n_na, n_nn, nd1) = (0, 1, 1)
                                    if ca == 1 and (n_na == 1 or e == 0): continue
                                    if cn == 1 and (n_nn == 1 or d0 <= nd1): continue
                                    if n_nn == 1 and d0 > nd1: continue
                                    if n_na == 1 and e == 0 and d0 > 0: continue
                                    let nd = max(d0, nd1)
                                    let has_a = Int(ca == 1 or n_na == 1)
                                    let has_n = Int(cn == 1 or n_nn == 1)
                                    ndp[nd, has_a, has_n] += v
                                    if ca == 0 and e == 1 and d0 == 0:
                                        ndp[nd, 1, has_n] -= v
        cur_dp = [[0..2, 0..2] (a, b) => ndp[d, a, b] for d in 0..sz]
    dp[cur] = cur_dp

dfs(V(1))
print(sum(row[a, b] for row in dp[V(1)] for a in 0..2 for b in 0..2))

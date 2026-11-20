// Codeforces 2237 I1: bottom-up tree DP counting canonical valid assignments
// by depth and (has_active, has_next) cut flags.
const MOD = 1_000_000_007
type Z = Mod<MOD>

@!multitest
let n: Int = read()
let s_in: Str = read()
let s = "0" + s_in
var ch: List<List<Int>> = List.filled(n, [])
for i in 0..n:
    let l: Int = read()
    for _ in 0..l:
        let j: Int = read()
        ch[i].push(j - 1)

var dp: List<List<[0..4]Z>> = List.filled(n, [])

fn dfs(cur: Int):
    var base: [0..4]Z = 0
    base[0] = 1
    var cur_dp: List<[0..4]Z> = [base]
    for nxt in ch[cur]:
        dfs(nxt)
        let nxt_dp = dp[nxt]
        let sz = max(cur_dp.len, nxt_dp.len + 1)
        let zero: [0..4]Z = 0
        var ndp: List<[0..4]Z> = List.filled(sz, zero)
        for d0 in 0..cur_dp.len:
            for d1 in 0..nxt_dp.len:
                for ca in 0..2:
                    for cn in 0..2:
                        let v_cur = cur_dp[d0][ca * 2 + cn]
                        if v_cur == 0: continue
                        for na in 0..2:
                            for nn in 0..2:
                                let v_nxt = nxt_dp[d1][na * 2 + nn]
                                if v_nxt == 0: continue
                                let v = v_cur * v_nxt
                                for e in 0..2:
                                    let ch_s = s[nxt]
                                    if ch_s != '?' and ch_s != Char('0'.ord + e): continue
                                    var n_na = na
                                    var n_nn = nn
                                    var nd1 = d1 + e
                                    if e == 1 and na == 1:
                                        n_na = 0
                                        n_nn = 1
                                        nd1 = 1
                                    if ca == 1 and n_na == 1: continue
                                    if cn == 1 and n_nn == 1: continue
                                    if ca == 1 and e == 0: continue
                                    if n_nn == 1 and d0 > nd1: continue
                                    if cn == 1 and d0 <= nd1: continue
                                    if n_na == 1 and e == 0 and d0 > 0: continue
                                    let nd = max(d0, nd1)
                                    let has_a = Int(ca == 1 or n_na == 1)
                                    let has_n = Int(cn == 1 or n_nn == 1)
                                    ndp[nd][has_a * 2 + has_n] += v
                                    if ca == 0 and e == 1 and d0 == 0:
                                        ndp[nd][2 + has_n] -= v
        cur_dp = ndp
    dp[cur] = cur_dp

dfs(0)
var ans: Z = 0
for row in dp[0]:
    for r in 0..4: ans += row[r]
print(ans)

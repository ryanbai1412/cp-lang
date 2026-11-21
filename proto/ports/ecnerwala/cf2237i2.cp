const MOD = 1_000_000_007

fn dfs(cur: Int, s: Str, ch: List<List<Int>>) -> List<Mod<MOD>>:
    var dp = [Mod<MOD>(1), Mod<MOD>(0), Mod<MOD>(0), Mod<MOD>(0), Mod<MOD>(1), Mod<MOD>(0)]
    for nxt in ch[cur]:
        let child = dfs(nxt, s, ch)
        let nh = max(dp.len / 6, child.len / 6 + 1)
        var ndp = [Mod<MOD>(0) for _ in 0..nh * 6]
        for (i0, vc) in dp.items if vc.v != 0:
            let (d0, ca, cn) = (i0 / 6, i0 % 6 / 2, i0 % 2)
            for (i1, vn) in child.items if vn.v != 0:
                let (d1, na, nn) = (i1 / 6, i1 % 6 / 2, i1 % 2)
                let v = vc * vn
                for e in 0..2:
                    let ch_e = if e == 0: '0' else: '1'
                    if s[nxt] != '?' and s[nxt] != ch_e: continue
                    let (n_na, n_nn, nd1) = if e == 1 and na != 0: (0, 1, 1) else: (na, nn, d1 + e)
                    if (ca != 0 and n_na != 0) or (cn != 0 and n_nn != 0) or (ca != 0 and e == 0): continue
                    if (n_nn != 0 and d0 > nd1) or (cn != 0 and d0 <= nd1): continue
                    if (n_na != 0 and d0 > 0) or (ca == 2 and na == 2): continue
                    let nd = max(d0, nd1)
                    let hn = if cn != 0 or n_nn != 0: 1 else: 0
                    if ca == 2 or na == 2:
                        if s[nxt] == '0':
                            if na == 2: ndp[nd * 6 + 4 + hn] += v
                        elif s[nxt] == '1':
                            if ca == 2 and na != 2: ndp[nd * 6 + 2 + hn] += v
                        elif ca == 2 and na != 2 and e == 1:
                            ndp[nd * 6 + 4 + hn] += v
                        continue
                    let ha = if ca != 0 or n_na != 0: 1 else: 0
                    ndp[nd * 6 + ha * 2 + hn] += v
                    if ca == 0 and e == 1 and d0 == 0: ndp[nd * 6 + 2 + hn] -= v
        dp = ndp
    dp

@!multitest
let n: Int = read()
let s_in: Str = read()
let s = "0" + s_in
var ch = List<List<Int>>()
for _ in 0..n:
    let l: Int = read()
    ch.push([read::<Int>() - 1 for _ in 0..l])
let root = dfs(0, s, ch)
var ans = Mod<MOD>(0)
for d in 0..root.len / 6:
    ans += root[d * 6] + root[d * 6 + 1] + root[d * 6 + 2] + root[d * 6 + 3]
print(ans)

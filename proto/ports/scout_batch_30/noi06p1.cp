// NOI 2006 Network Charge: 2^m leaves on a complete binary tree each have an
// initial bit arr[i], flip cost cost[i], and pairwise traffic flow[i][j].
// Choose final leaf bits minimizing flip costs plus traffic charges whenever
// two leaves' LCA majority differs from a leaf's chosen bit.
// Bottom-up DP over tree node u (1..2n), ancestor majority bitmask par (d bits),
// and count of 1-leaves cnt in u's subtree; leaf costs use row prefix sums of
// flow[u][..].
// original: corpus/submissions/5032743.cpp
let m: 1..=10 = read()
let n = 1 << m

let arr: [0..n]Int = read()
let cost: [0..n]Int = read()
var flow: [0..n, 0..n]Int = 0
for i in 0..n:
    for j in (i + 1)..n:
        let w: Int = read()
        flow[i, j] = w
        flow[j, i] = w

var psum: [0..n, 0..=n]Int = 0
for i in 0..n:
    for j in 0..n:
        psum[i, j + 1] = psum[i, j] + flow[i, j]

const INF = 4000000000000000000
let sz = n * 2
var dp: [0..sz, 0..sz]Int = INF

for d in (0..=m).rev:
    let u_lo = 1 << d
    let u_hi = 1 << (d + 1)
    let par_hi = 1 << d
    let cnt_hi = 1 << (m - d + 1)
    for u in u_lo..u_hi:
        for par in 0..par_hi:
            for cnt in 0..cnt_hi:
                let state = (cnt << d) | par
                if d == m:
                    var val = 0
                    let cu = u - n
                    if (arr[cu] ^ cnt) != 0:
                        val += cost[cu]
                    for pd in (0..m).rev:
                        if (((par >> pd) & 1) ^ cnt) != 0:
                            let ls = m - pd
                            let half = 1 << (ls - 1)
                            var p = (cu >> ls) << ls
                            if ((cu >> (ls - 1)) & 1) == 0:
                                p += half
                            val += psum[cu, p + half] - psum[cu, p]
                    dp[u, state] = val
                else:
                    var best = INF
                    let csz = 1 << (m - d - 1)
                    let lo_a = max(0, cnt - csz)
                    let hi_a = min(csz, cnt)
                    let flag_bit = if cnt > csz: 1 else: 0
                    let cflag = (flag_bit << d) | par
                    for acnt in lo_a..=hi_a:
                        let bcnt = cnt - acnt
                        let cand = dp[u * 2, (acnt << (d + 1)) | cflag] + dp[u * 2 + 1, (bcnt << (d + 1)) | cflag]
                        best min= cand
                    dp[u, state] = best

var ans = INF
for s in 0..sz:
    ans min= dp[1, s]
print(ans)

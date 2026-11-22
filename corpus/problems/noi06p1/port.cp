// NOI 2006 Network Charge (CP Lang 0.2.2): bottom-up tree DP using Tropical
// Optionals `[0..sz, 0..sz]?Int = none` + `min=` (zero `INF` sentinel).
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

let sz = n * 2
var dp: [0..sz, 0..sz]?Int = none

for d in (0..=m).rev:
    let (u_lo, u_hi) = (1 << d, 1 << (d + 1))
    let par_hi = 1 << d
    let cnt_hi = 1 << (m - d + 1)
    for u in u_lo..u_hi:
        for par in 0..par_hi:
            for cnt in 0..cnt_hi:
                let state = (cnt << d) | par
                if d == m:
                    let cu = u - n
                    var val = if (arr[cu] ^ cnt) != 0: cost[cu] else: 0
                    for pd in (0..m).rev:
                        if (((par >> pd) & 1) ^ cnt) != 0:
                            let ls = m - pd
                            let half = 1 << (ls - 1)
                            var p = (cu >> ls) << ls
                            if ((cu >> (ls - 1)) & 1) == 0: p += half
                            val += psum[cu, p + half] - psum[cu, p]
                    dp[u, state] = val
                else:
                    var best: ?Int = none
                    let csz = 1 << (m - d - 1)
                    let cflag = (Int(cnt > csz) << d) | par
                    for acnt in max(0, cnt - csz)..=min(csz, cnt):
                        let bcnt = cnt - acnt
                        best min= dp[u * 2, (acnt << (d + 1)) | cflag] + dp[u * 2 + 1, (bcnt << (d + 1)) | cflag]
                    dp[u, state] = best

var ans: ?Int = none
for s in 0..sz: ans min= dp[1, s]
print(ans!)

// Runnable port (proto/cp.py lib-run) of etsr23r2p2.cp
// original: corpus/submissions/5494433.cpp

const CAP = 254

let tcs: Int = read()
for _ in 0..tcs:
    let s: Str = read()
    let target: Int = read()
    let n = s.len
    domain Pos = 0..n
    domain Val = 0..=CAP

    var num: [Pos][Pos]Int = [Pos] _ => [Pos] _ => 0
    for li in (0..n).rev:
        let l = Pos(li)
        num[l][l] = s[li] - '0'
        for ri in (li + 1)..n:
            num[l][Pos(ri)] = min(CAP + 1, num[l][Pos(ri - 1)] * 10 + (s[ri] - '0'))

    var dp1: [Pos][Pos][Val]?Int = [Pos] _ => [Pos] _ => [Val] _ => none
    for li in (0..n).rev:
        let l = Pos(li)
        for ri in li..n:
            let r = Pos(ri)
            let base = Val(min(num[l][r], CAP))
            dp1[l][r][base] = 0
            for mi in li..ri:
                let mid = Pos(mi)
                let nxt = Pos(mi + 1)
                let lhs = num[l][mid]
                for vi in 0..=CAP:
                    let c = dp1[nxt][r][Val(vi)]
                    if c != none:
                        let nv = Val(min(lhs * vi, CAP))
                        let cand = 1 + c!
                        let cur = dp1[l][r][nv]
                        if cur == none or cand < cur!:
                            dp1[l][r][nv] = cand

    var dp2: [Pos][Pos][Val]?Int = [Pos] _ => [Pos] _ => [Val] _ => none
    for li in (0..n).rev:
        let l = Pos(li)
        for ri in li..n:
            let r = Pos(ri)
            for vi in 0..=CAP:
                let v = Val(vi)
                dp2[l][r][v] = dp1[l][r][v]
            for mi in li..ri:
                let mid = Pos(mi)
                let nxt = Pos(mi + 1)
                for va in 0..=CAP:
                    let ca = dp1[l][mid][Val(va)]
                    if ca == none: continue
                    for vb in 0..=CAP:
                        let cb = dp2[nxt][r][Val(vb)]
                        if cb == none: continue
                        let nv = Val(min(va + vb, CAP))
                        let cand = ca! + 1 + cb!
                        let cur = dp2[l][r][nv]
                        if cur == none or cand < cur!:
                            dp2[l][r][nv] = cand

    let ans = dp2[Pos.first][Pos.last][Val(target)]
    if ans == none: print(-1)
    else: print(ans!)

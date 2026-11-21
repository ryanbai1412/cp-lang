// ETSR '23 Round 2 P2 — Minimum '+' and '*' Expression Interval DP (CP Lang v2 spec)
// original: corpus/submissions/5494433.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Insert the fewest '+' and '*' operators into digit string `s` so the
// expression evaluates to `target` (standard precedence, values capped at 254).

const CAP = 254

@multitest
let (s, target): (Str, 0..=CAP) = read()
let n = s.len
domain Pos = 0..n
domain Val = 0..=CAP

var num: [Pos, Pos]Int = 0
for l in Pos.rev:
    num[l, l] = s[l.to_idx] - '0'
    for r in l.r..:
        num[l, r] = min(CAP + 1, num[l, (r - 1)!] * 10 + (s[r.to_idx] - '0'))

// dp1[l, r, v]: fewest '*' to form product value v on s[l..=r].
// dp2[l, r, v]: fewest '+' and '*' to form sum-of-products value v on s[l..=r].
var dp1: [Pos, Pos, Val]?Int = none
var dp2: [Pos, Pos, Val]?Int = none

for l in Pos.rev:
    for r in l..:
        dp1[l, r, Val(min(num[l, r], CAP))] min= 0
        for mid in l..r:
            let next = mid + 1
            for v in Val:
                if let c = dp1[next, r, v]:
                    dp1[l, r, Val(min(num[l, mid] * v.to_idx, CAP))] min= 1 + c

for l in Pos.rev:
    for r in l..:
        for v in Val:
            dp2[l, r, v] min= dp1[l, r, v]
        for mid in l..r:
            let next = mid + 1
            for va in Val:
                if let ca = dp1[l, mid, va]:
                    for vb in Val:
                        if let cb = dp2[next, r, vb]:
                            dp2[l, r, Val(min(va.to_idx + vb.to_idx, CAP))] min= ca + 1 + cb

print(dp2[Pos.first, Pos.last, Val(target)], none: "-1")

// expect: ok, output "0 1 6"
// rule: 07.1.6
// On `Int` keys `pre[k]` is the cut: the fold of the first `k`, with `n + 1` entries (rebuild 8, H-INT-KEY-PREFIX, decision 349).
let p = [1, 2, 3].scan(+)
print(p[0], p[1], p[3])

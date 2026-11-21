use std.seq.{inverse}

@!multitest
let n: Int = read()
domain I = 0..n
domain V = 1..=n
let vals: [V]Int = read()
let a: [I]V = read()
let b: [I]V = read()
let inv_b = b.inverse()

var tot = 0
var deltas: Map<Int, Int> = Map()
for z in I:
    let cur = a[z]
    let v = vals[cur]
    let pos = inv_b[cur].to_idx
    tot += v
    deltas[pos] -= v
    while let cur_val = deltas.get(pos), cur_val < 0:
        if let (nxt_k, nxt_v) = deltas.first_gt(pos):
            let d = min(-cur_val, nxt_v)
            if nxt_v == d: deltas.remove(nxt_k) else: deltas[nxt_k] -= d
            if cur_val + d == 0: deltas.remove(pos) else: deltas[pos] += d
        else:
            deltas.remove(pos)

print(tot + sum(v for (_, v) in deltas))

@!multitest
let n: Int = read()
let vals: [0..n]Int = read()
let a = [0..n] _ => read::<Int>() - 1
let b = [0..n] _ => read::<Int>() - 1
var inv_b: [0..n]Int = 0
for i in 0..n: inv_b[b[i]] = i

var tot = 0
var deltas: Map<Int, Int> = Map()
for z in 0..n:
    let cur = a[z]
    let v = vals[cur]
    let pos = inv_b[cur]
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

@!multitest
let n: Int = read()
let vals: [0..n]Int = read()
let a: List<Int> = [read::<Int>() - 1 for _ in 0..n]
let b: List<Int> = [read::<Int>() - 1 for _ in 0..n]
var inv_b: [0..n]Int = 0
for i in 0..n: inv_b[b[i]] = i

var tot = 0
var deltas: Map<Int, Int> = Map()
for z in 0..n:
    let cur = a[z]
    let v = vals[cur]
    let pos = inv_b[cur]
    tot += v
    deltas[pos] = (deltas.get(pos) ?? 0) - v
    while (deltas.get(pos) ?? 0) < 0:
        if let nxt = deltas.first_gt(pos):
            let cur_val = deltas.get(pos)!
            let d = min(-cur_val, nxt.1)
            let new_cur = cur_val + d
            let new_nxt = nxt.1 - d
            if new_nxt == 0: deltas.remove(nxt.0) else: deltas[nxt.0] = new_nxt
            if new_cur == 0:
                deltas.remove(pos)
                break
            else:
                deltas[pos] = new_cur
        else:
            deltas.remove(pos)
            break
for (_, v) in deltas: tot += v
print(tot)

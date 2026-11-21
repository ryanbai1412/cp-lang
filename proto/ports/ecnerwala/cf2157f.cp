// Codeforces 2157 F: choose the optimal number of layers d and max weight by
// binomial counts, then emit layer-ordered jump operations recursively.
let n: Int = read()
var (best_d, best_cost, max_w, max_w_cnt) = (1, n * (n - 1) / 2, n - 1, 1)

for d in 2..=n:
    var (rem, cnt, cost, w) = (n, 1, (d - 1) * 1000, 0)
    if cost >= best_cost: break
    while rem > 0:
        if w > 0: cnt = cnt * (d + w - 1) / w
        cnt min= rem
        cost += cnt * w
        if cnt == rem: break
        rem -= cnt
        w += 1
    if cost < best_cost:
        (best_cost, best_d, max_w, max_w_cnt) = (cost, d, w, rem)

let d = best_d
var layer_ops: [0..d]List<(Int, Int)> = []

fn go(cur_layer: Int, cur_w: Int, start: Int) -> Int:
    let cl = if cur_w == max_w: d else: cur_layer
    if cl == d:
        if cur_w == max_w:
            max_w_cnt -= 1
            if max_w_cnt == 0: max_w -= 1
        return start - 1
    var cur = start
    for i in (0..=(max_w - cur_w)).rev:
        if cur < 0: break
        if cur != start: layer_ops[cl].push((cur, start - cur))
        cur = go(cl + 1, cur_w + i, cur)
    cur

go(0, 0, n - 1)
let ops = [op for layer in layer_ops.rev for op in layer]

print(ops.len)
for (x, l) in ops: print(x + 1, l)

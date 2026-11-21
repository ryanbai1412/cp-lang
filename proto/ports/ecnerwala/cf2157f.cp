let n: Int = read()
var best_d = 1
var best_cost = n * (n - 1) / 2
var max_w = n - 1
var max_w_cnt = 1

for d in 2..=n:
    var rem = n
    var cnt = 1
    var cost = (d - 1) * 1000
    if cost >= best_cost: break
    var w = 0
    while rem > 0:
        if w > 0: cnt = cnt * (d + w - 1) / w
        cnt min= rem
        cost += cnt * w
        if cnt == rem: break
        rem -= cnt
        w += 1
    if cost < best_cost:
        best_cost = cost
        best_d = d
        max_w = w
        max_w_cnt = rem

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
let ops = [op for layer in (0..d).rev for op in layer_ops[layer]]

print(ops.len)
for (x, l) in ops: print(x + 1, l)

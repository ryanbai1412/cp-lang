// Codeforces 2157 F: Git Gud.
// Merge `n` positions using `d` layers of jumps (`1 <= d <= n`). Each layer
// switch costs `LAYER_COST = 1000` (`(d - 1) * 1000` total), and each position
// assigned a `d`-tuple of layer counts summing to `w` costs `w`.
//   - At `d = 1`, weights `0..n` cost `n * (n - 1) / 2`.
//   - For `d >= 2`, there are `C(d + w - 1, w)` tuples of weight `w`. Picking
//     the `n` smallest weights yields `Plan(d, cost, max_w, max_w_cnt)`;
//     minimize `cost` over `d`.
// Walking the `n` tuples in reverse lex order emits each jump `(cur, start - cur)`
// into its layer; printing layers in reverse order gives the final sequence.

use std.math.{choose}

const LAYER_COST = 1000

struct Plan(d: Int, cost: Int, max_w: Int, max_w_cnt: Int)

let n: 1..=250_000 = read()

fn plan_for(d: Int) -> Plan:
    var rem = n
    var cost = (d - 1) * LAYER_COST
    for w in 0..n:
        let take = min(rem, choose(d + w - 1, w))
        cost += take * w
        if take == rem: return Plan(d, cost, w, take)
        rem -= take
    Plan(d, cost, n - 1, 0)

var best = Plan(1, n * (n - 1) / 2, n - 1, 1)
for d in 2..=n:
    if (d - 1) * LAYER_COST >= best.cost: break
    let cand = plan_for(d)
    if cand.cost < best.cost: best = cand

let d = best.d
var layer_ops: [0..d]List<(Int, Int)> = []
var lim_w = best.max_w
var lim_cnt = best.max_w_cnt

var cur: ?Int = some(n - 1)

fn go(cur_layer: Int, cur_w: Int):
    let cl = if cur_w == lim_w: d else: cur_layer
    if cl == d:
        if cur_w == lim_w:
            lim_cnt -= 1
            if lim_cnt == 0: lim_w -= 1
        if let pos = cur:
            cur = if pos > 0: some(pos - 1) else: none
        return
    let start = cur ?? return
    for i in (0..=(lim_w - cur_w)).rev:
        let pos = cur ?? break
        if pos != start: layer_ops[cl].push((pos, start - pos))
        go(cl + 1, cur_w + i)

go(0, 0)
let ops = [op for layer in layer_ops.rev for op in layer]
print(ops.len)
for (x, l) in ops: print(x + 1, l)

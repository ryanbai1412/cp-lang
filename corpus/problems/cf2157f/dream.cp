// Codeforces 2157 F: Git Gud.
// Merge `n` positions using `d` layers of jumps (`1 <= d <= n`). Each layer
// switch costs `LAYER_COST = 1000` (`(d - 1) * 1000` total), and assigning each
// position a distinct `d`-tuple `c = (c_0, ..., c_{d-1})` of non-negative layer
// counts with weight `w = sum(c)` costs `w` (the total jump count for that
// position):
//   - At `d = 1`, weights `0..n` cost `n * (n - 1) / 2`.
//   - For `d >= 2`, there are `C(d + w - 1, w)` tuples of weight `w`. Choosing
//     the `n` smallest-weight tuples minimizes total cost; `plan_for(d)` sums
//     each weight layer until `n` tuples are taken. Once `(d - 1) * LAYER_COST`
//     reaches `best.cost`, no larger `d` can improve on `best`.
//
// Reverse-lex merge order: sort the `n` chosen `d`-tuples in reverse
// lexicographic order, placing them at positions `n` down to `1`. Truncating
// the last weight class `max_w` keeps the chosen set downward-closed because
// every tuple of weight `< max_w` is already included in full, so decrementing
// any coordinate of a chosen weight-`max_w` tuple lands in a strictly smaller
// weight class. Grouping `placed` by prefix `t[..cl]` at each layer `cl` sends
// every member after the group's first position `start` jumping forward to
// `start`, merging all `n` positions in exact budget.

use std.math.{choose}

const LAYER_COST = 1000

struct Plan(d: Int, cost: Int, max_w: Int, max_w_cnt: Int)

let n: 1..=250_000 = read()

// Number of d-tuples of non-negative integers summing to w: C(d + w - 1, w).
fn tuples_of_weight(d: Int, w: Int) -> Int = choose(d + w - 1, w)

fn plan_for(d: Int) -> Plan:
    var rem = n
    var cost = (d - 1) * LAYER_COST
    for w in 0..n:
        let take = min(rem, tuples_of_weight(d, w))
        cost += take * w
        if take == rem: return Plan(d, cost, w, take)
        rem -= take
    Plan(d, cost, n - 1, 0)

fn best_plan() -> Plan:
    var res = Plan(1, n * (n - 1) / 2, n - 1, 1)
    for d in 2..=n:
        if (d - 1) * LAYER_COST >= res.cost: break
        let cand = plan_for(d)
        if cand.cost < res.cost: res = cand
    res

// Enumerate the `n` cheapest d-tuples in reverse lex order.
// Walk the cheapest tuples in order, emitting each layer's jumps as the walk
// leaves a branch, so no tuple is ever stored.
fn emit_jumps(plan: Plan) -> List<(Int, Int)>:
    var layer_ops: [0..plan.d]List<(Int, Int)> = []
    var next_pos = n
    var lim_w = plan.max_w
    var max_w_left = plan.max_w_cnt
    fn walk(layer: Int, used_w: Int):
        if layer == plan.d or used_w == lim_w:
            if used_w == plan.max_w:
                max_w_left -= 1
                if max_w_left == 0: lim_w -= 1
            next_pos -= 1
            return
        let start = next_pos
        for c in (0..=lim_w - used_w).rev:
            if next_pos == 0: break
            if next_pos != start: layer_ops[layer].push((next_pos, start - next_pos))
            walk(layer + 1, used_w + c)
    walk(0, 0)
    [op for layer in layer_ops.rev for op in layer]

let ops = emit_jumps(best_plan())
print(ops.len)
for (x, l) in ops: print(x, l)

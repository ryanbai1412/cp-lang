// Codeforces 2157 F: Git Gud.
// Merge `n` positions using `d` layers of jumps (`1 <= d <= n`). Each layer
// switch costs `LAYER_COST = 1000` (`(d - 1) * 1000` total), and assigning each
// position a distinct `d`-tuple `c = (c_0, ..., c_{d-1})` of non-negative layer
// counts with weight `w = sum(c)` costs `w` (the total jump count for that
// position):
//   - At `d = 1`, weights `0..n` cost `n * (n - 1) / 2`.
//   - For `d >= 2`, there are `C(d + w - 1, w)` tuples of weight `w`. Choosing
//     the `n` smallest-weight tuples minimizes total cost; `plan_for(d)` sums
//     each weight layer until `n` tuples are taken.
//
// Reverse-lex merge order: sort the `n` chosen `d`-tuples in reverse
// lexicographic order, placing them at positions `n - 1` down to `0`. Because
// the chosen set is downward-closed, at each layer `cl` every tuple with
// `c[cl] > 0` jumps forward to the first position `start` sharing the same
// prefix `(c_0, ..., c_{cl-1})`. Emitting these jumps per layer and printing
// layers from `d - 1` down to `0` merges all `n` positions in exact budget.

use std.math.choose

const LAYER_COST = 1000

struct Plan(d: Int, cost: Int, max_w: Int, max_w_cnt: Int)

let n: 1..=250_000 = read()

// Number of d-tuples of non-negative integers summing to w: C(d + w - 1, w).
fn tuples_of_weight(d: Int, w: Int) -> Int = choose(d + w - 1, w)

fn plan_for(d: Int) -> Plan:
    var rem = n
    var cost = (d - 1) * LAYER_COST
    var w = 0
    while true:
        let take = min(rem, tuples_of_weight(d, w))
        cost += take * w
        if take == rem: return Plan(d, cost, w, take)
        rem -= take
        w += 1

let best = (1..=n)
    .take_while(d => (d - 1) * LAYER_COST < n * (n - 1) / 2 or d == 1)
    .map(plan_for)
    .min(by: p => p.cost)!

// Enumerate the `n` cheapest d-tuples in reverse lex order.
fn cheapest_tuples(plan: Plan) -> List<List<Int>>:
    var out: List<List<Int>> = []
    var cur = List.filled(plan.d, 0)
    var max_w_left = plan.max_w_cnt
    fn gen(layer: Int, used_w: Int):
        if layer == plan.d:
            if used_w == plan.max_w:
                if max_w_left == 0: return
                max_w_left -= 1
            out.push(cur.clone())
            return
        for c in (0..=plan.max_w - used_w).rev:
            cur[layer] = c
            gen(layer + 1, used_w + c)
    gen(0, 0)
    out

let tuples = cheapest_tuples(best)
domain Pos = 1..=n
fn pos_of(idx: Int) -> Pos = Pos(n - idx)

var layer_ops: [0..best.d]List<(Pos, Int)> = []
for cl in 0..best.d:
    var group_start = pos_of(0)
    for (idx, t) in tuples.items:
        let pos = pos_of(idx)
        if idx > 0 and tuples[idx - 1][..cl] != t[..cl]:
            group_start = pos
        if pos != group_start:
            layer_ops[cl].push((pos, group_start.ord - pos.ord))

let ops = [op for layer in layer_ops.rev for op in layer]
print(ops.len)
for (x, l) in ops: print(x, l)

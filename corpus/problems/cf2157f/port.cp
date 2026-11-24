// Source: Codeforces 2157F (Git Gud), https://codeforces.com/contest/2157/problem/F
// limits: 2000 ms, 256 MB
// Problem: unknown skill s in 1..=n (n = 4 or 250000). A mission (difficulty y, length l) costs
//   l, plus 1000 if y is above the previous mission's difficulty, and adds l to s when y == s.
//   Print at most 1e6 missions, total cost <= 1e6, that bring every possible s to >= n.
// Solution: a run of non-increasing difficulties pays only lengths, so use d descending passes
//   ((d - 1) * 1000 for the climbs). Every start skill gets a distinct d-tuple of jumps per pass
//   and the lengths total the tuples' weights, so take the n lightest tuples (C(d + w - 1, w) of
//   weight w), trying d = 2, 3, ... until (d - 1) * 1000 reaches the best cost (d = 1 costs
//   n(n - 1)/2). A DFS over the chosen tuples in reverse lex order emits each pass's jumps
//   (difficulty x, length start - x); passes print last first. O(n) per tried d.
// Bounds: n: 1..=250_000


const LAYER_COST = 1000

struct Plan(d: Int, cost: Int, max_w: Int, max_w_cnt: Int)

let n: 1..=250_000 = read()

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

// Stream the reverse-lex prefix tree directly instead of materializing
// `List<List<Int>>` + `chunk_by` slices (dream:55-84), which takes 6.5s at -O2
// for n = 250_000 due to millions of small vector copies (see friction).
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

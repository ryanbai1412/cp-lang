// let: V = \sqrt n; L = \sqrt n; R = \sqrt n; F = \sqrt n; P = \sqrt n
// fixed: c
use std.flow.{min_cost_circulation}
let n: Int = read()
let k = n.isqrt
domain V = 0..k
domain E = 0..k * k
let pad = k * k - 2 * k
var s = 0
// The same chain as a closure: v_i before v_{i-1}, profit 1 on each v_i but v_0, which costs k;
// the padding prerequisites point upward, so no path gets shorter.
let before = [V] v => do:
    var b = if v.ord + 1 < k: [V(v.ord + 1)] else: []
    if v.ord > 0: b += [V(j % v.ord) for j in 0..k - 2]
    b
let profit = [V] v => if v.ord == 0: -k else: 1
s += max_closure(before, profit: v => profit[v]).value // timed: flow.max_closure
let bounds: [E](u: V, v: V, lo: Int, hi: Int) = [E] e => do:
    let i = e.ord
    if i < k - 1: (u: V(0), v: V(i + 1), lo: 1, hi: 1)
    elif i < 2 * k - 2: (u: V(i - k + 2), v: V(i - k + 1), lo: 0, hi: k)
    else: (u: V((i * 7919) % k), v: V((i * 104729 + 1) % k), lo: 0, hi: 0)
let priced: [E](u: V, v: V, lo: Int, hi: Int, cost: Int) = [E] e => (u: bounds[e].u, v: bounds[e].v, lo: bounds[e].lo, hi: bounds[e].hi, cost: 0)
s += (circulation(bounds) ?? []).len // timed: flow.circulation
s += (max_bounded_flow(bounds, V(1), V(0)))!.value // timed: flow.max_bounded_flow
s += (min_bounded_flow(bounds, V(1), V(0)))!.value // timed: flow.min_bounded_flow
s += (min_cost_circulation(priced))!.cost // timed: flow.min_cost_circulation
print(s)

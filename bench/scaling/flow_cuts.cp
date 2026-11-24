// let: V = \sqrt n; L = \sqrt n; R = \sqrt n; F = \sqrt n; P = \sqrt n; flow = n
// fixed: c
let n: Int = read()
let k = n.isqrt
domain V = 0..k
domain E = 0..k * k
var s = 0
// A complete graph with random weights: each of gomory_hu's max flows ends in O(1) phases of
// O(E) each, so one flow is O(n).
let weighted: [E](u: V, v: V, w: Int) = [E] e => (u: V(e.ord / k), v: V(e.ord % k), w: (e.ord * 7919) % 97 + 1)
s += gomory_hu(weighted).len // timed: flow.gomory_hu
s += global_min_cut(weighted).w // timed: flow.global_min_cut
for _ in 0..5:
    s += max_weight_general_matching(weighted).total // timed: flow.max_weight_general_matching
print(s)

// expect: ok, output "5 -1 0 9"
// rule: std.matrix.Mat.new std.matrix.Mat.product
// BOI 2017 Toll shape: each gap between layers of `k` places is a min-plus matrix, `None`
// meaning no road, and a route is their product, folded in a SegTree over the gaps.
domain Gap = 0..3
domain Slot = 0..2
let roads = [(0, 2, 4), (1, 3, 2), (0, 3, 9), (2, 4, 1), (3, 5, 7), (4, 6, 3)]
var cost: [Gap, Slot, Slot]?Int = None
for (a, b, t) in roads:
    let c = cost[Gap(a / 2), Slot(a % 2), Slot(b % 2)]
    cost[Gap(a / 2), Slot(a % 2), Slot(b % 2)] = Some(min(c ?? t, t))
let step: [Gap]Mat<?Int> = [Gap] g => Mat([Slot, Slot] (i, j) => cost[g, i, j], over: Semiring(min, +))
let st = SegTree<Gap, Mat<?Int>>(Mat<?Int>.product(2, over: Semiring(min, +)), step)
let route = (a: Int, b: Int) => st.fold(Gap(a / 2)..Gap(b / 2)).get(a % 2, b % 2) ?? -1
let empty = Mat<?Int>.filled(2, 2, None, over: Semiring(min, +))
print(route(0, 4), route(1, 2), (empty * st.monoid.id).get(0, 0) ?? 0, route(0, 3))

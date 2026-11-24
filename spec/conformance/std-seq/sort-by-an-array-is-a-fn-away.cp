// expect: E0302
// has: "x => w[x]"
// A sort key is a fn: an array `w: [V]Int` isn't one, and the help and fix read it in one,
// `by: x => w[x]` (decision 634; the compiler arm that took `by: w` is gone).
domain V = 0..3
let w: [V]Int = [3, 1, 2]
var order = V.sorted(by: x => x)
order.sort(by: w) // <- here
print(order)

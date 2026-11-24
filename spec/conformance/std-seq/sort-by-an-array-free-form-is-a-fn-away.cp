// expect: E0302
// has: "x => w[x]"
// The free form reads the array in a fn the same way: `sort(mut order, by: x => w[x])` (decision 634).
domain V = 0..3
let w: [V]Int = [3, 1, 2]
var order = V.sorted(by: x => x)
sort(mut order, by: w) // <- here
print(order)

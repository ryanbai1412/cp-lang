// expect: ok, output "3000000000"
// rule: 03.1.7
// `+` is not closed on `0..=1e9`, so a `Window(+)` over such cells folds `Int`, like `SegTree(+, a)` (rebuild round 29 H04).
let a: [3](0..=1000000000) = [1000000000, 1000000000, 1000000000]
var w = Window(+)
for x in a: w.push(x)
print(w.all!)

// expect: ok, output "3 1"
// rule: 02.5.1
// A `let` pattern destructures the tuple that `!` unwraps.
let xs = [(3, 1), (4, 2)]
let (x, i) = xs.first!
print(x, i)

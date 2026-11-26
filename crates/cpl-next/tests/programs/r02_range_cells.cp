// expect: R0201
// A list of a range type: `push` and a write to a cell check what they store, `get` gives an `Int`.
var x: 0..=10 = 5
let y = x
var xs: List<0..=10> = List.new()
xs.push(y + 1)
print(xs.get(0) * 100)
xs[0] = y + 6 // <- here

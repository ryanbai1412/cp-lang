// expect: E0602
// rule: 05.7.1 std.bag.SortedBag.range
// `range(r)` is `lends`, so the bag stays lent while the view lives, as `xs` does under `xs.filter(p)`: adding to it first is an error.
var b = SortedBag.of([1, 3])
let v = b.range(..)
b.add(5) // <- here
print(v.len)

// expect: E0602
// rule: 05.7.1 std.bag.SortedBag.range
// A `for` over `b.range(..)` lends the bag until the loop ends, as a `for` over `xs.filter(p)` lends `xs` (adversarial round 46).
var b = SortedBag.of([1, 3])
for x in b.range(..):
    b.add(x) // <- here
print(b.len)

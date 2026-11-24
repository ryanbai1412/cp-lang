// expect: R0401 (checked)
// rule: std.bag.SortedBag.add_many
var b = SortedBag.of([1])
b.add_many(5, -1) // <- here
print(b.len)

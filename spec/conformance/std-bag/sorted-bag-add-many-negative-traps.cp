// expect: R0401 at 2 (checked)
// rule: std.bag.SortedBag.add_many
var b = SortedBag.of([1])
b.add_many(5, -1)
print(b.len)

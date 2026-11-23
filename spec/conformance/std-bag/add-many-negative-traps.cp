// expect: R0401 at 3 (checked)
var b = SortedBag.of([1])
b.add_many(5, -1)
print(b.len)

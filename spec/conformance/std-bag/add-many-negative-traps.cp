// expect: R0401 (checked)
var b = SortedBag.of([1])
b.add_many(5, -1) // <- here
print(b.len)

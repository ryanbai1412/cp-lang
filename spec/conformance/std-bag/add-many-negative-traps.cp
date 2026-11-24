// expect: E0902
var b = SortedBag.of([1])
b.add_many(5, -1) // <- here
print(b.len)

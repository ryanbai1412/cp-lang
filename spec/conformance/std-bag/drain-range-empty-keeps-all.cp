// expect: ok, output "0 3"
var b = SortedBag.of([1, 3, 5])
print(b.drain_range(4..5).collect().len, b.len)

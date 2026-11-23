// expect: ok, output "0 1"
var b = SortedBag.of([3])
print(b.remove_all(2), b.len)

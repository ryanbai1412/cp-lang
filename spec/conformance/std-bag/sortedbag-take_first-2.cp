// expect: ok, output "YES"
var b = SortedBag.of([1])
print(yes(b.take_first(x => x >= 3) == None and b.len == 1))

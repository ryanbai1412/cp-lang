// expect: ok, output "YES"
var b = SortedBag.of([1, 4, 4, 6])
print(yes(b.take_first(x => x >= 3) == Some(4) and b.to_list() == [1, 4, 6]))

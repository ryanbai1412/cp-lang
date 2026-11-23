// expect: ok, output "YES"
var b = SortedBag.of([3, 3, 1])
print(yes(b.pop_last() == Some(3) and b.to_list() == [1, 3]))

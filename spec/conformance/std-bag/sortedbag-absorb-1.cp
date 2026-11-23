// expect: ok, output "YES"
var a = SortedBag.of([1, 3])
var b = SortedBag.of([3, 2])
a.absorb(mut b)
print(yes(a.to_list() == [1, 2, 3, 3] and b.is_empty))

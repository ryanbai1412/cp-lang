// expect: ok, output "YES"
var s = SortedSet.of([1, 2])
let t = s.take()
print(yes(s.is_empty and t.to_list() == [1, 2]))

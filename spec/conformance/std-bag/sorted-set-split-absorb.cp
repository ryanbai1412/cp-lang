// expect: ok, output "1 2\n1 2 3 5"
var s = SortedSet.of([1, 3, 5])
var hi = s.split_at(3)
print(s.len, hi.len)
s.add(2)
s.absorb(mut hi)
print(s.to_list())

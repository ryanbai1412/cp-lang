// expect: ok, output "2 4 1\n1 2 3\n2 4 1 1 4 2"
// rule: 07.4.17
// A slice's `items` and `rev` are the ordinary std views over it: both walk backwards, and `to_list` copies either.
let a = [3, 1, 4, 2, 5]
let s = a[1..4]
print([x for (k, x) in s.items.rev])
print([k for (k, x) in s.items])
print(s.rev.to_list(), s.to_list())

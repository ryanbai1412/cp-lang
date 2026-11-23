// expect: ok, output "3 1 2\n3,1,2\n1 a\n2 b\nx 1 2.5000000000|\n2 1 3\nabc\n1 2\n3\n0.3333333333\n-4 1"
// rule: 10.3.1
// Mixed print forms (sep:, end:, pairs, Char lists, nested lists, F64, floor division) print the same in the interpreter and C++.
let xs = [3, 1, 2]
let ps = [(1, 'a'), (2, 'b')]
print(xs)
print(xs, sep: ",")
print(ps, sep: "\n")
print("x", 1, 2.5, end: "")
print("|")
print(xs.rev, sep: "-")
print(['a', 'b', 'c'])
print([[1, 2], [3]], sep: "\n")
print(1.0 / 3.0)
print(-7 / 2, -7 % 2)

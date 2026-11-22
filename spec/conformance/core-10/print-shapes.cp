// expect: ok, output "ab\ncd\n1 2\n3\n3 1 2 3"
// rule: 10.3.6
let g = "ab cd".parse<[2, 2]Char>()!
print(g)
print([[1, 2], [3]])
let xs = [1, 2, 3]
print(xs.len, xs)

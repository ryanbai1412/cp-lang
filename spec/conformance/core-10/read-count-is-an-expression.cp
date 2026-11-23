// expect: ok, output "2"
// input: "3\n1 2\n2 3\n"
// rule: 10.2.3
// The count of `[e]T` may be any `Int` expression, as for the n - 1 edges of a tree.
let n = read<Int>()
let e = read<[n - 1](u: Int, v: Int)>()
print(e.len)

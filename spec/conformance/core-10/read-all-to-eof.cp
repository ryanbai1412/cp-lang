// expect: ok, output "3 9"
// rule: 10.2.7
// input: "4 5\n\n0\n"
let xs = read_all<Int>()
print(xs.len, xs.sum())

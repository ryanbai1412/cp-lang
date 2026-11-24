// expect: R0201
// rule: 02.2.1
// input: "-1\n"
fn g(self: Int, i: 0..) -> Int = self + i
let n = read<Int>()
print(3.g(n)) // <- here

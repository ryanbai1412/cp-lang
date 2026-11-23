// expect: ok, output "3"
// input: "5\n3\n"
// rule: 02.10.2
// A local may carry a range type with runtime bounds, read and checked in one line.
let n = read<Int>()
var i: 1..=n = read()
print(i)

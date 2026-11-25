// expect: R0201
// rule: R02.10.2
// input: "3\n"
// Open run-time range lo..: only the lower end is checked.
let n: Int = read()
var x: n.. = 100
x = n - 1  // <- here
print(x)

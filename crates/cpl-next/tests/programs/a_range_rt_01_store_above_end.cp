// expect: R0201
// rule: R02.10.2, R03.1.4
// input: "5\n"
// R02.10.2: a var with a run-time range checks each assignment.
let n: Int = read()
var i: 1..=n = 1
i = n + 1  // <- here
print(i)

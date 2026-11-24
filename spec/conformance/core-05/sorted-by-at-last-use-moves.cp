// expect: ok, output "8"
// input: "3\n3 7\n1 8\n2 9\n"
// rule: 05.2.2
// A `let` sorted at its last use moves into the sort, `.sorted(by:)` as `.sorted()`.
let n: 2..=10 = read()
let xs: [n](t: Int, x: Int) = read()
print(xs.sorted(by: o => o.t).first!.x)

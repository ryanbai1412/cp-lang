// expect: E0503
// rule: 01.8.2
// has: "`,`"
// A comprehension's clauses are joined with `,`, not a second `for`.
let ps = [(i, j) for i in 0..3 for j in 0..2] // <- here
print(ps.len)

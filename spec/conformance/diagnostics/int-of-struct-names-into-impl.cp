// expect: E0302
// rule: 04.2.28
// has: "impl Into<Int>"
// With no conversion and no single field to name, the help names the impl to write (decision 390).
struct P:
    a: Int
    b: Int
print(Int(P(a: 1, b: 2))) // <- here

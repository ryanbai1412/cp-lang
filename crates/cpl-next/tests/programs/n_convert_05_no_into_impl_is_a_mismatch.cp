// expect: E0302
// rule: 04.2.28
// has: "impl Into<Int> for Cell"
// `Int(x)` converts by `x`'s own `Into<Int>` impl; with none it is a type mismatch whose help names the impl.
struct Cell(v: Int)
print(Int(Cell(3))) // <- here

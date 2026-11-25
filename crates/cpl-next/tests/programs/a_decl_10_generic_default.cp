// expect: ok, output "2"
// rule: R08
// Generic struct with a default type argument.
struct Cell<T = Int>:
    v: T
let c: Cell = Cell(2)
print(c.v)

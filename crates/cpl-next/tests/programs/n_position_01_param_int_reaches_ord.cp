// expect: ok, output "2\n4 5"
// A position's integer on a type parameter is its `Domain` impl's `ord`, checked wherever an
// instance runs it: `adj4` over a named product domain steps `R` and `C` as parameters
// (R04.2.15, R04.3.3).
domain R = 0..3
domain C = 0..3
domain Cell = (R, C)
fn nb(p: Cell) -> yields Cell:
    for q in p.adj4: yield q
fn next<D: Domain>(p: D) -> Int = Int(p + 1)
let ps = [q for q in nb((R(0), C(0)))]
print(ps.len)
print(next(R(2)) + 1, next(C(1)) + 3)

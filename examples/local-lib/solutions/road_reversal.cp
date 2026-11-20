// Each of m roads can be kept or reversed; road i must not point the same
// way as road j for every listed conflict. Print one valid choice or "NO".
// A 2-SAT instance: variable i is "road i is reversed".
use lib.graph.two_sat.{TwoSat, Lit}

let (m, k): (Int, Int) = read()
domain Road = 1..=m
var sat = TwoSat<Road>()
for _ in 0..k:
    let (a, b): (Int, Int) = read()
    sat.differ(Lit(Road(a), true), Lit(Road(b), true))
if let plan = sat.solve():
    print("YES")
    print([if plan[r]: 1 else: 0 for r in Road])
else:
    print("NO")

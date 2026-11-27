// expect: ok, output "4 2\n1 3"
// rule: R04.2.1 R07.2.1
// A domain's `count` getter takes no arguments, so `V.count(pred)` is `Seq`'s count of the positions
// that pass; `D.count` and `D.count(pred)` on a bounded parameter choose the same way.
domain V = 1..=4
fn both<D: Domain>(x: D) -> (Int, Int) = (D.count, D.count(v => Int(v) % 2 == 1))
print(V.count, V.count(v => Int(v) > 2))
let (a, b) = both(V(1))
print(a - 3, b + 1)

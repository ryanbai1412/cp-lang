// expect: ok, output "8 11 31\n12 9\n6"
// rule: R04.2.10 R04.2.11
// A range written as a product's factor is a domain of its own, sized where the product is
// declared, at the top level or in a fn.
let n = 3
domain State = (0..=n, 0..2)
var best: [State]Int = [State] s => 0
for s in State: best[s] = s.0.ord * 10 + s.1.ord
print(best.len, best[State.nth(3)], best[State.nth(best.len - 1)])
fn local(k: Int) -> Int:
    domain P = (0..k, 1..=3)
    let t: [P]Int = [P] p => 1
    t.len
print(local(4), local(n))
domain R = 0..2
domain Q = (R, 5..=6)
print(Int(Q.nth(3)._1))

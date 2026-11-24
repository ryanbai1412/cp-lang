// expect: ok, output "8 11 31\n12 9"
// rule: 04.2.10
// A range written as a factor of a product domain is an index space of its own, sized where the
// product is defined, at the top level or in a fn.
let n = 3
domain State = (0..=n, 0..2)
var best: [State]Int = [State] s => 0
for s in State: best[s] = s.0.ord * 10 + s.1.ord
print(best.len, best[State.nth(3)], best[State.nth(best.len - 1)])
fn local(k: Int) -> Int:
    domain P = (c: 0..k, d: 1..=3)
    let t: [P]Int = [P] p => 1
    t.len
print(local(4), local(n))

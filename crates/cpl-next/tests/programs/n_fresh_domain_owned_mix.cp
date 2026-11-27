// expect: E0302
// rule: R02.9.3
// `u.P` is the domain `u` owns: a position of `t`'s is not one of it.
struct Tagged<V, P>:
    at: [P]V
fn make<V: Domain>(from: V, k: Int) -> Tagged<V, _>:
    domain O = 0..k
    Tagged<V, O>(at: [O] o => V.nth(from.ord + o.ord))
domain W = 0..10
let t = make(W(1), 3)
let u = make(W(0), 3)
let r: u.P = t.P.nth(0) // <- here
print(Int(u.at[r]))

// expect: ok, output "3\n1"
// rule: R02.9.3
// A value's own domain is found through its binding: `t.P` reads as a type too, and a nearer
// binding of the same name owns a domain of its own.
struct Tagged<V, P>:
    at: [P]V
fn make<V: Domain>(from: V, k: Int) -> Tagged<V, _>:
    domain O = 0..k
    Tagged<V, O>(at: [O] o => V.nth(from.ord + o.ord))
domain W = 0..10
let t = make(W(1), 3)
let p: t.P = t.P.nth(2)
print(Int(t.at[p]))
fn inner():
    let t = make(W(0), 2)
    let q: t.P = t.P.nth(1)
    print(Int(t.at[q]))
inner()

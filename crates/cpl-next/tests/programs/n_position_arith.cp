// expect: ok, output "4 2 -1 7\n3 YES\n1 1"
// rule: R04.3.3
// `v + k`, `v - k` and `v - w` on positions are `Int`s of their integers, on a declared domain
// and on a parameter used as one; a struct's `const` is read through its name.
domain V = 2..9
let v = V(3)
print(v + 1, v - 1, v - V(4), V.last! - 1)
fn below<R>(p: R) -> ?R = R.at(p + 1)
fn gap<D: Domain>(a: D, b: D) -> Int = b - a
print(gap(V(2), V(5)), yes(below(V(8)) == None))
struct P(x: Int):
    const one: P = P(1)
print(P.one.x, Dir4.right.delta.1)

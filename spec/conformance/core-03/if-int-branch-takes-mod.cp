// expect: ok, output "3 0 999999664"
// rule: 02.10.1
// Siblings unify (R02.10.1) and an `Int` converts to `Mod<P>` (R04.4.28), so an `if` with one `Int` branch and one `Mod<P>` branch is a `Mod<P>`, in either order.
type Z = Mod<1_000_000_007>
let cb = Comb<Z>(10)
let a = if true: 0 else: cb.choose(3, 1)
let b = if false: cb.choose(3, 1) else: 0
let c = if true: 1_000_000_000 else: cb.choose(3, 1)
print(cb.choose(3, 1) + a, b, c * 1_000_000_000 * 1_000_000_000)

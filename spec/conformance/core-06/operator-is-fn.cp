// expect: ok, output "5 6"
// rule: 06.1.6
struct Frac:
    n: Int
    d: Int
fn (a: Frac) + (b: Frac) -> Frac = Frac(n: a.n * b.d + b.n * a.d, d: a.d * b.d)
let c = Frac(n: 1, d: 2) + Frac(n: 1, d: 3)
print(c.n, c.d)

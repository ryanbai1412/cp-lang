// expect: L0603
type A = Mod<1_000_000_007>
let coef = ["1", "2"]
let under_a = coef.map(s => s.parse<A>()!) // <- here
let roots = (1..=3).filter(x => under_a.eval(A(x)) == A(0)).collect()
print(roots.len)

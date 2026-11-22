// noip14p6 Solve the Equation: the coefficients have 10^4 digits, so evaluate the polynomial under
// two primes; a root is a root under both, and a false root under both is vanishingly unlikely.
type A = Mod<1_000_000_007>
type B = Mod<998_244_353>
let (n, m) = read<(1..=100, 1..=1_000_000)>()
let coef = read<[0..=n]Str>()
let under_a = coef.map(s => s.parse<A>()!).collect()
let under_b = coef.map(s => s.parse<B>()!).collect()
let roots = (1..=m).filter(x => under_a.eval(A(x)) == A(0) and under_b.eval(B(x)) == B(0)).collect()
print(roots.len)
print(roots, sep: "\n")

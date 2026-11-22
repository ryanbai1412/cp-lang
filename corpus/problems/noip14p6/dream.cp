// NOIP 2014 Solving Equations: integer roots in 1..=m of a polynomial whose coefficients have
// up to 10^4 digits. Evaluate modulo two primes; a root vanishes under both.
let (n, m) = read (Int, Int)
let digits = read [n + 1]Str
let c1 = digits.map(s => s.parse<Mod<1_000_000_007>>()).list()
let c2 = digits.map(s => s.parse<Mod<998_244_353>>()).list()
let roots = (1..=m).filter(x => c1.eval(x) == 0 and c2.eval(x) == 0).list()
print(roots.len)
print(roots, sep: "\n")

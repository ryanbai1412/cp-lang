// NOIP 2014 Solving Equations: integer roots in 1..=m of a polynomial whose coefficients have
// up to 10^4 digits. Evaluate modulo a large prime; a root makes the value vanish.
type Z = Mod<2305843009213693951>
let n: 1..=100 = read()
let m: 1..=1e6 = read()
fn residue(s: Str) -> Z:
    let v = s.fold((v, c) => if c == '-': v else: v * Z(10) + Z(c - '0'), Z(0))
    if s.starts_with("-"): -v else: v
let coef = [residue(s) for s in read<[n + 1]Str>()]
let roots = [x for x in 1..=m if coef.eval(Z(x)) == Z(0)]
print(roots.len)
print(roots, sep: "\n")

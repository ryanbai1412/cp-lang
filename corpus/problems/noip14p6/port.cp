// Source: NOIP 2014 Senior D2T3, Solve the Equation (not on DMOJ; Luogu P2312)
// limits: 1000 ms, 128 MB
// Problem: given n <= 100, m < 1e6 and integers a_0..a_n with |a_i| <= 10^10000, a_n != 0,
//   print the number of integer roots of a_0 + a_1 x + ... + a_n x^n = 0 in 1..=m, then
//   the roots in increasing order, one per line.
// Solution: read each coefficient modulo the prime 2^61 - 1 and test every x in 1..=m by
//   Horner's rule (`eval`); x is reported when the value is 0, which a nonzero value
//   passes only by bad luck (about n / 2^61 per x). O(n m).
// original: corpus/submissions/5445671.cpp
let n: 1..=100 = read()
let m: 1..=1e6 = read()

// a short signed number, for stress tests
fn random_number() -> List<Char>:
    var s: List<Char> = []
    if rand(0, 1) == 0: s.push('-')
    for _ in 0..rand(1, 3): s.push('0' + rand(0, 9))
    s

// The numbers are far too long to hold, so keep each one modulo a large prime.
type Z = Mod<2305843009213693951>
fn residue(s: [domain S]Char) -> Z:
    let v = s.fold((v, c) => if c == '-': v else: v * Z(10) + Z(c - '0'), Z(0))
    if s[S.first!] == '-': -v else: v
let coef = [residue(read(gen: random_number)) for _ in 0..=n]

let roots = [x for x in 1..=m if coef.eval(Z(x)) == Z(0)]
print(roots.len)
for x in roots: print(x)

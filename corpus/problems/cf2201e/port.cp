// Codeforces 2201 E: ABBA Counting.
// With m = n / 2, S = A + B + B + A says exactly that the second half of S is
// its first half X rotated left by |A|. So count pairs (X, shift), then
// correct for strings hit by several shifts.
// If X has least period p, rotations repeat every p and S is hit by m / p
// shifts. Let ways(len) count pairs (X, shift mod len) with X of period
// len, for each len = m / f. S is counted there iff p | len, len / p times;
// weighting ways(m / f) by c(f) = prod over primes r | f of (1 - r) makes S
// count sum over f | q of c(f) * q / f = 1 for q = m / p, since
// c = (mu * id) * 1 and (mu * id) * id is the identity of Dirichlet
// convolution.
// For one len, fold each half onto len residues (a clash leaves no X). A
// shift pairs residue j of the first half with j + shift of the second; it
// is consistent iff every pair of known letters agrees, and then each pair
// with no known letter is free. Over all shifts at once: correlating the
// +1/-1 letter signs gives agree - disagree, correlating known flags gives
// the number of known pairs.
use std.math.{divisors, factor}
use std.ntt.{convolve}

type Z = Mod<998_244_353>

@!multitest
let n: 2..=400_000 = read()
let t: Str = read()
let m = n / 2
let pow2 = [0..=m] i => Z(2) ** i

// The half t[from..from + m] folded onto `len` residues, or none on a clash.
fn fold(from: Int, len: Int) -> ?List<?Char>:
    var s: List<?Char> = [none for _ in 0..len]
    for i in 0..m if t[from + i] != '?':
        if let c = s[i % len], c != t[from + i]: return none
        s[i % len] = t[from + i]
    s

fn sign(c: ?Char) -> Int = if let c = c: (if c == 'a': 1 else: -1) else: 0
fn known(c: ?Char) -> Int = if let _ = c: 1 else: 0

fn ways(len: Int) -> Z:
    let x = fold(0, len) ?? return 0
    let y = fold(m, len) ?? return 0
    let agree = convolve(x.map(sign), y.map(sign).reversed(), cyclic: true)
    let pairs = convolve(x.map(known), y.map(known).reversed(), cyclic: true)
    let fixed = x.count(c => c is some) + y.count(c => c is some)
    sum(pow2[len - fixed + pairs[i]] for i in 0..len if agree[i] == pairs[i])

print(sum(ways(m / f) * prod(Z(1 - r) for (r, _) in factor(f)) for f in divisors(m)))

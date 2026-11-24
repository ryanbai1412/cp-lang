// Source: Codeforces 2201E (ABBA Counting), https://codeforces.com/contest/2201/problem/E
// limits: 3500 ms, 256 MB
// Problem: a string t of even length n <= 4e5 over 'a', 'b', '?'. Count fillings S of the '?'
//   that can be written S = A + B + B + A for some (possibly empty) strings A, B, mod 998244353.
//   Sum n <= 4e5; multitest.
// Solution: with m = n / 2, S = ABBA iff its second half is the first half X rotated by |A|,
//   so count pairs (X, shift) and fix overcounting: X of least period p is hit by m / p shifts.
//   ways(len) counts pairs (X of period len, shift mod len) by folding each half onto len
//   residues; a shift is consistent iff known letters agree, and each unknown pair is free.
//   Cyclic NTT correlations of the +-1 signs and of the known flags give, for all shifts at
//   once, agreements and known pairs. Answer = sum over f | m of ways(m / f) * prod (1 - r)
//   over primes r | f (mu * id Moebius weights): O(m d(m) + sigma(m) log m).
// Bounds: n: 2..=400_000

type Z = Mod<998_244_353>

@!multitest
let n: 2..=400_000 = read()
let t: Str = read()
let m = n / 2
let pow2 = [0..=m] i => Z(2) ** i

// The half t[from..from + m] folded onto `len` residues, or none on a clash.
fn fold(from: Int, len: Int) -> ?List<?Char>:
    var s: List<?Char> = [None for _ in 0..len]
    for start in 0..m / len:
        for i in 0..len:
            let c = t[from + start * len + i]
            if c == '?': continue
            if let d = s[i], d != c: return None
            s[i] = c
    s

fn sign(c: ?Char) -> Z = if let c = c: (if c == 'a': Z(1) else: Z(-1)) else: Z(0)
fn known(c: ?Char) -> Z = if let _ = c: Z(1) else: Z(0)

fn ways(len: Int) -> Z:
    let x = fold(0, len) ?? return 0
    let y = fold(m, len) ?? return 0
    let agree = convolve(x.map(sign), y.map(sign).reversed(), cyclic: true)
    let pairs = convolve(x.map(known), y.map(known).reversed(), cyclic: true)
    let fixed = x.count(c => c is some) + y.count(c => c is some)
    sum(pow2[len - fixed + pairs[i].v] for i in 0..len if agree[i] == pairs[i])

print(sum(ways(m / f) * prod(Z(1 - r) for (r, _) in factor(f)) for f in divisors(m)))

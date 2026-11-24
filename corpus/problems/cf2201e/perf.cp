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

// Tuned: a fold onto `len` residues is the fold of any multiple's fold, so the divisors of m are
// folded largest first, each from its nearest folded multiple, instead of each from the full
// halves: the folding work drops from m·d(m) to about σ(m), and the NTTs are what is left.

type Z = Mod<998_244_353>

@!multitest
let n: 2..=400_000 = read()
let t: Str = read()
let m = n / 2
let pow2 = [0..=m] i => Z(2) ** i

fn letters(from: Int) -> List<?Char>:
    var s: List<?Char> = [None for _ in 0..m]
    for k in 0..m if t[from + k] != '?':
        s[k] = t[from + k]
    s

// `src` folded onto `len` residues (len divides src.len), or none on a clash.
fn fold(src: ?List<?Char>, len: Int) -> ?List<?Char>:
    let cells = src ?? return None
    var s: List<?Char> = [None for _ in 0..len]
    var i = 0
    for c in cells:
        if let c = c:
            if let d = s[i], d != c: return None
            s[i] = c
        i = if i + 1 == len: 0 else: i + 1
    s

fn sign(c: ?Char) -> Z = if let c = c: (if c == 'a': Z(1) else: Z(-1)) else: Z(0)
fn known(c: ?Char) -> Z = if let _ = c: Z(1) else: Z(0)

fn ways(x: List<?Char>, y: List<?Char>) -> Z:
    let len = x.len
    let agree = convolve(x.map(sign), y.map(sign).reversed(), cyclic: true)
    let pairs = convolve(x.map(known), y.map(known).reversed(), cyclic: true)
    let fixed = x.count(c => c is some) + y.count(c => c is some)
    sum(pow2[len - fixed + pairs[i].v] for i in 0..len if agree[i] == pairs[i])

let lens = divisors(m).reversed()
let halves: (x: ?List<?Char>, y: ?List<?Char>) = (x: letters(0), y: letters(m))
var folded: List<(x: ?List<?Char>, y: ?List<?Char>)> = []
var answer = Z(0)
for (k, len) in lens.items:
    var parent = halves
    var j = k - 1
    while j >= 0:
        if lens[j] % len == 0:
            parent = folded[j]
            break
        j -= 1
    let here = (x: fold(parent.x, len), y: fold(parent.y, len))
    if let x = here.x:
        if let y = here.y:
            answer += ways(x, y) * prod(Z(1 - r) for (r, _) in factor(m / len))
    folded.push(here)
print(answer)

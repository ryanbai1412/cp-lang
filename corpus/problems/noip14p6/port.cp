// NOIP 2014 P6, Solve the Equation: find every x in 1..=m with
// a[0] + a[1] x + ... + a[n] x^n = 0, where each a[i] has up to 10^4 digits.
// original: corpus/submissions/5445671.cpp
let n: 1..=100 = read()
let m: 1..=1e6 = read()

// a short signed number, for stress tests
fn random_number() -> List<Char>:
    var s: List<Char> = []
    if rand(0, 1) == 0: s.push('-')
    for _ in 0..rand(1, 3): s.push('0' + rand(0, 9))
    s

// The numbers are far too long to hold, so keep each one modulo two primes.
// A root is a root modulo both; the other way round is a safe bet.
let primes = [1000000007, 998244353]
var a: [0..2, 0..=n]Int = 0
for i in 0..=n:
    let s: [domain S]Char = read(gen: random_number)
    for t in 0..2:
        var v = 0
        for c in s:
            if c != '-': v = (v * 10 + (c - '0')) % primes[t]
        a[t, i] = if s[S.first!] == '-': -v % primes[t] else: v

fn is_root(x: Int, t: Int) -> Bool:
    var y = 0
    for i in (0..=n).rev: y = (y * x + a[t, i]) % primes[t]  // Horner
    y == 0

let roots = [x for x in 1..=m if is_root(x, 0) and is_root(x, 1)]
print(roots.len)
for x in roots: print(x)

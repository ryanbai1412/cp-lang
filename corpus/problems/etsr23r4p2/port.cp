// Source: ETSR '23 R4 P2 (SHOI 2017, Encounters Are Greetings), https://dmoj.ca/problem/etsr23r4p2
// Problem: 1 <= n, q <= 5e4 numbers 0 <= a_i <= 1e8, a modulus 1 <= p <= 1e8 and a base 1 <= c <= 1e8.
//   "0 l r" replaces each a_i in l..=r by c^(a_i); "1 l r" prints the sum of a_l..=a_r mod p.
// Solution: by the generalized Euler theorem, c^e mod m depends on e mod phi(m) (plus phi(m)
//   once e >= phi(m)), so a tower of raisings settles modulo p after as many steps as the
//   totient chain p, phi(p), ..., 1 is long (O(log p)). Each leaf precomputes its value after s
//   raisings for every s up to that length, a node keeps those sums, and a range raise shifts
//   them by one, saturating at the settled value. O((n + q) log n log p).
// original: corpus/submissions/5529698.cpp
// Bounds: n, q: 1..=5e4; p, base: 1..=1e8; a: 0..=1e8; kind: 0..=1; l, r: 1..=n

let (n, q, p, base): (1..=5e4, 1..=5e4, 1..=1e8, 1..=1e8) = read()
type Z = Mod<p>
domain I = 1..=n
let a: [I]0..=1e8 = read()

var moduli = [p]
while moduli.last! > 1:
    moduli.push(phi(moduli.last!))
domain Slot = 0..=moduli.len

fn modulus(depth: Int) -> Int = moduli[min(depth, moduli.len - 1)]

// base^e mod modulus(depth), and whether the true power reaches it; `big` says e itself
// reached modulus(depth + 1) = phi(modulus(depth)), so the true exponent is e + that.
// base^e mod modulus(d) as low[d][e % step] * high[d][e / step]; e < 2 * modulus(d + 1)
let step = 1 << 15
let low = [[pow_mod(base, e, modulus(d)) for e in 0..step] for d in 0..moduli.len]
let high = [
    [
        pow_mod(base, h * step, modulus(d)) for h in 0..=2 * modulus(d + 1) / step
    ] for d in 0..moduli.len
]

fn power(e: Int, big: Bool, depth: Int) -> (big: Bool, val: Int):
    let m = modulus(depth)
    let exp = if big: e + modulus(depth + 1) else: e
    var (reached, acc) = (m <= 1, 1)
    if base > 1:
        for _ in 0..exp:  // base^exp >= m within log2(m) <= 27 factors
            acc *= base
            if acc >= m:
                reached = true
                break
    let d = min(depth, moduli.len - 1)
    (big: reached, val: low[d][exp % step] * high[d][exp / step] % m)

// x raised s times, mod modulus(depth).
fn tower(s: Int, x: Int, depth: Int) -> (big: Bool, val: Int):
    if s == 0: return (big: x >= modulus(depth), val: x % modulus(depth))
    let below = tower(s - 1, x, depth + 1)
    power(below.val, below.big, depth)

let sums = Monoid<[Slot]Z>(id: [Slot] _ => Z(0), op: (x, y) => [Slot] s => x[s] + y[s])
let raise = Action<[Slot]Z, Int>(
    id: 0,
    compose: (+),
    apply: (k, x, _) => [Slot] s => x[Slot.at(Int(s) + k) ?? Slot.last!],
    over: sums,
    commutes: true,
)
var st = SegTree(sums, [I] i => [Slot] s => Z(tower(Int(s), a[i], 0).val), act: raise)
for _ in 0..q:
    let (kind, l, r): (0..=1, I, I) = read()
    if kind == 0: st.apply(l..=r, 1)
    else: print(st.fold(l..=r)[Slot.first!])

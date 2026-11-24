// ETSR 2023 R4 P2: range "a[i] = base^a[i]" and range sum mod p. A power tower mod p settles
// after as many raisings as the totient chain p, φ(p), …, 1 is long, so every leaf precomputes
// its value after s raisings for each s, a node keeps those sums, and a range raise shifts them,
// saturating at the settled value.
let (n, q, p, base): (1..=5e4, 1..=5e4, 1..=1e8, 1..=1e8) = read()
domain I = 1..=n
let a: [I]0..=1e8 = read()
var chain = [p]
while chain.last! > 1: chain.push(phi(chain.last!))
domain Step = 0..=chain.len
fn modulus(depth: Int) -> Int = chain[min(depth, chain.len - 1)]

// The least exponent with base^e >= chain[i]; at or past it the generalized Euler theorem keeps
// the exponent mod φ but with φ added, so a value carries whether it got there.
fn least_reaching(m: Int) -> Int:
    if m <= 1: return 0
    if base == 1: return 64
    var (e, x) = (0, 1)
    while x < m: (e, x) = (e + 1, x * base)
    e
let enough = chain.map(least_reaching).collect()
// base^e mod chain[i] by table: every exponent here is below 2e8 < 2^28, two 14-bit halves.
fn powers(m: Int, x: Int) -> List<Int>:
    var t = [1 % m]
    for _ in 1..1 << 14: t.push(t.last! * x % m)
    t
let low = chain.map(m => powers(m, base % m)).collect()
let high = chain.map(m => powers(m, pow_mod(base, 1 << 14, m))).collect()
fn raise(below: (big: Bool, val: Int), depth: Int) -> (big: Bool, val: Int):
    let e = if below.big: below.val + modulus(depth + 1) else: below.val
    let i = min(depth, chain.len - 1)
    (big: e >= enough[i], val: low[i][e % (1 << 14)] * high[i][e >> 14] % chain[i])
fn tower(s: Int, x: Int, depth: Int) -> (big: Bool, val: Int) =
    if s == 0: (big: x >= modulus(depth), val: x % modulus(depth))
    else: raise(tower(s - 1, x, depth + 1), depth)

type Z = Mod<p>
let sums = Monoid<[Step]Z>(id: [Step] _ => Z(0), op: (x, y) => [Step] s => x[s] + y[s])
let shift = Action<[Step]Z, Int>(id: 0, compose: (+), over: sums, commutes: true,
                                 apply: (k, x, _) => [Step] s => x[Step.at(Int(s) + k) ?? Step.last!])
var st = SegTree(sums, [I] i => [Step] s => Z(tower(Int(s), a[i], 0).val), act: shift)
for (kind, l, r) in read<[q](0..=1, I, I)>():
    if kind == 0: st.apply(l..=r, 1) else: print(st.fold(l..=r)[Step.first!])

// ETSR 2023 R4 P2: range "a[i] = base^a[i]" and range sum mod p. A power tower mod p settles
// after as many raisings as the totient chain p, φ(p), …, 1 is long, so every leaf precomputes
// its value after s raisings for each s, a node keeps those sums, and a range raise shifts them,
// saturating at the settled value.
use std.math.{Exp, pow_exp}

let (n, q, p, base): (1..=5e4, 1..=5e4, 1..=1e8, 1..=1e8) = read()
domain I = 1..=n
let a: [I]0..=1e8 = read()
var chain = [p]
while chain.last! > 1: chain.push(phi(chain.last!))
domain Step = 0..=chain.len
fn modulus(depth: Int) -> Int = chain[min(depth, chain.len - 1)]

// x raised s times (base ** ... ** base ** x with s bases), kept for modulus(depth)
fn tower(s: Int, x: Int, depth: Int) -> Exp =
    if s == 0: Exp.of(x, modulus(depth))
    else: pow_exp(base, tower(s - 1, x, depth + 1), modulus(depth))

type Z = Mod<p>
let sums = Monoid<[Step]Z>(id: [Step] _ => Z(0), op: (x, y) => [Step] s => x[s] + y[s])
let shift = Action<[Step]Z, Int>(
    id: 0,
    compose: (+),
    over: sums,
    commutes: true,
    apply: (k, x, _) => [Step] s => x[s.shift_saturating(k)],
)
var st = SegTree(sums, [I] i => [Step] s => Z(tower(Int(s), a[i], 0).v), act: shift)
for (kind, l, r) in read<[q](0..=1, I, I)>():
    if kind == 0: st.apply(l..=r, 1) else: print(st.fold(l..=r)[Step.first!])

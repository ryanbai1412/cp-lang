// etsr23r4p2: "a[i] = base^a[i] on a range" and "sum on a range, mod p". A power tower settles
// modulo p after as many steps as the totient chain p, phi(p), ..., 1 is long, so each node keeps,
// for every number s of further raisings up to that length, the sum its range would have; a
// range raise shifts those sums along by one, and the shifts commute.
let (n, q, p, base) = read<(1..=50_000, 1..=50_000, 1..=100_000_000, 1..=100_000_000)>()
type Z = Mod<p>
domain I = 1..=n
let a = read<[I]Int>()
var moduli = [p]
while moduli.last! > 1:
    moduli.push(phi(moduli.last!))
domain Slot = 0..=moduli.len
// base^e modulo m, and whether the true power reaches m (generalized Euler needs the flag).
fn power(e: Int, big: Bool, m: Int) -> (big: Bool, val: Int):
    let exp = if big: e + m else: e
    var (val, reached, acc) = (1 % m, false, 1)
    for _ in 0..min(exp, 64):
        acc = min(acc * base, m)
        reached = reached or acc >= m
    (big: reached or (exp > 64 and base > 1), val: pow_mod(base, exp, m))
// x raised s times, modulo moduli[depth] (the last modulus once the chain runs out).
fn tower(s: Int, x: Int, depth: Int) -> (big: Bool, val: Int):
    let m = moduli[min(depth, moduli.len - 1)]
    if s == 0:
        return (big: x >= m, val: x % m)
    let below = tower(s - 1, x, depth + 1)
    power(below.val, below.big, m)
let sums = Monoid(id: [Slot] _ => Z(0), op: (x: [Slot]Z, y: [Slot]Z) => [Slot] s => x[s] + y[s])
let raise = Action<[Slot]Z, Int>.new(
    apply: (k: Int, x: [Slot]Z, len: Int) => [Slot] s => x[Slot.at(Int(s) + k) ?? Slot.last!],
    over: sums,
)
var st = SegTree(sums, [I] i => [Slot] s => Z(tower(Int(s), a[i], 0).val), act: raise)
for _ in 0..q:
    let (kind, l, r) = read<(0..=1, I, I)>()
    if kind == 0:
        st.apply(l..=r, 1)
    else:
        print(st.fold(l..=r)[Slot.first!])

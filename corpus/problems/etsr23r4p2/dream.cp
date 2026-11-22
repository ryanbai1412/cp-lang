// ETSR 2023 R4 P2: range "a[i] = base^a[i]" and range sum mod p. A power tower mod p settles
// after as many raisings as the totient chain p, φ(p), …, 1 is long, so every leaf precomputes
// its value after s raisings for each s, a node keeps those sums, and a range raise shifts them.
let (n, q, p, base) = read (Int, Int, Int, Int)
domain I = 1..=n
let a = read [I]Int
let chain = iterate(p, m => phi(m)).take_while(m => m > 1).chain([1]).list()
domain Step = chain.keys
fn tower(s: Int, x: Int, depth: Step) -> Exp = if s == 0: Exp(x, mod: chain[depth])
    else: Exp.pow(base, tower(s - 1, x, depth.next ?? depth), mod: chain[depth])   // generalized Euler
var st = SegTree(+, [I] i => [Step] s => Mod<p>(tower(Int(s), a[i], Step.first!).value),
                 act: shift_saturating)
for (kind, l, r) in read [q](Int, I, I):
    if kind == 1: st.apply(l..=r, 1) else: print(st.fold(l..=r)[Step.first!])

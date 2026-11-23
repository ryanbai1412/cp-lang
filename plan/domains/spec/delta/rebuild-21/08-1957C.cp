// CF 1957C How Does the Rook Move?: m free lines; choose 2j of them for mirrored pairs,
// (2j)! / j! ordered pairings, the rest go on the diagonal.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>(300_000)

@!multitest
let n = read<1..=3e5>()
let k = read<0..=n>()
let moves = read<[k](r: 1..=n, c: 1..=n)>()
let m = n - moves.map(mv => if mv.r == mv.c: 1 else: 2).sum()
print((0..=m / 2).map(j => comb.choose(m, 2 * j) * comb.fact(2 * j) * comb.inv_fact(j)).sum())

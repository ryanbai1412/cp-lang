// coci20c6p3 Anagramistica: choose a subset of words with exactly k similar (anagram) pairs. Only
// the sizes of anagram classes matter: taking c words from a class of s gives choose(s, c) ways
// and c(c - 1) / 2 pairs, a knapsack over the classes.
type Z = Mod<1_000_000_007>
let (n, k) = read<(1..=2_000, 0..=2_000)>()
let sizes = read<[n]Str>().tally(by: s => s.sorted()).values.collect()
domain G = sizes.keys
let comb = Comb<Z>(n)
@memo(array)
fn ways(g: G.cuts, left: 0..=k) -> Z:
    if g == G.cuts.last!:
        return if left == 0: 1 else: 0
    let s = sizes[g.r!]
    (0..=s).take_while(c => c * (c - 1) / 2 <= left)
        .map(c => comb.choose(s, c) * ways(g.next!, left - c * (c - 1) / 2)).sum()
print(ways(G.cuts.first!, k))

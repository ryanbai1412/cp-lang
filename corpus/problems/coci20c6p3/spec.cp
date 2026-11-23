// coci20c6p3 Anagramistica: count the subsets of n distinct words with exactly k anagram pairs.
// Only the sizes of anagram classes matter: taking c words of a class of s gives choose(s, c)
// ways and choose2(c) pairs, a knapsack over the classes on the pair count.
type Z = Mod<1_000_000_007>
let (n, k) = read<(1..=2_000, 0..=2_000)>()
let comb = Comb<Z>(n)
var ways: [0..=k]Z = [0..=k] j => if j == 0: 1 else: 0
for s in read<[n]Str>().tally(by: w => w.sorted()).values:
    ways = [0..=k] j => (0..=s).take_while(c => choose2(c) <= j).sum(
        of: c => comb.choose(s, c) * ways[j - choose2(c)]
    )
print(ways[k])

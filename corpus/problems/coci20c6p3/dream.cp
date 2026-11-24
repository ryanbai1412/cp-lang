// COCI 2020/21 C6 P3: choose a subset of the words so that exactly k pairs in it are anagrams.
// Count the subsets mod 1e9+7. Group words by letter multiset; taking c words from a group of s
// gives C(c, 2) pairs in C(s, c) ways: a knapsack over groups.
type M = Mod<1_000_000_007>
let n: 1..=2000 = read()
let k: 0..=2000 = read()
let words: [n]Str = read()
let comb = Comb<M>(n)
var ways: [0..=k]M = [0..=k] j => M(if j == 0: 1 else: 0)
for s in words.tally(by: w => w.sorted()).values:
    ways = [0..=k] j => (0..=s).take_while(c => choose2(c) <= j)
        .map(c => comb.choose(s, c) * ways[j - choose2(c)]).sum()
print(ways[k])

# COCI 2020/21 C6 P3: choose a subset of the words so that exactly k pairs in it are anagrams.
# Count the subsets mod 1e9+7. Group words by letter multiset; taking c words from a group of s
# gives C(c, 2) pairs in C(s, c) ways: a knapsack over groups.
type M = Mod<1_000_000_007>
let k = read Int
let groups = read [n] Str.tally(by: w => w.sorted()).values
var ways: [0..=k]M = [0..=k] j => if j == 0: 1 else: 0
for s in groups:
    ways = [0..=k] j => (0..=s).take_while(c => c * (c - 1) / 2 <= j)
        .sum(of: c => binom(s, c) * ways[j - c * (c - 1) / 2])
print(ways[k])

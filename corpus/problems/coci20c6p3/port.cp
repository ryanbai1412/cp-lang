// Source: COCI 2020/2021 Contest 6 #3 (Anagramistica), https://dmoj.ca/problem/coci20c6p3
// limits: 1000 ms, 512 MB
// Problem: n <= 2000 distinct words of <= 10 lowercase letters and k <= 2000. Count the subsets
//   of the words with exactly k pairs of anagrams, mod 1e9+7.
// Solution: anagrams form classes (keyed by letter counts) and only class sizes matter: taking
//   t words of a class of size s makes t(t - 1)/2 pairs in C(s, t) ways. A knapsack over classes
//   on the pair count 0..=k, stopping a class once t(t - 1)/2 passes k: O(n k).
const P = 1000000007
type Z = Mod<P>
let n: 1..=2000 = read()
let k: 0..=2000 = read()
domain Letter = 'a'..='z'
domain Pairs = 0..=k

// words that are anagrams of each other form a class; only class sizes matter
var class_size: Map<[Letter]Int, Int> = []
for _ in 0..n:
    let word: Str = read()
    var letters: [Letter]Int = 0
    for c in word: letters[Letter(c)] += 1
    class_size[letters] += 1

let comb = Comb<Z>(n)

// ways[p]: choices among the classes seen so far that make exactly p pairs
var ways: [Pairs]Z = Z(0)
ways[Pairs.first!] = Z(1)
for (_, size) in class_size:
    var next: [Pairs]Z = Z(0)
    for p in Pairs:
        for take in 0..=size:
            let made = Pairs.at(p.ord + take * (take - 1) / 2) else: break
            next[made] += comb.choose(size, take) * ways[p]
    ways = next
print(ways[Pairs.last!])

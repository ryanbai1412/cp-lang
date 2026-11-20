// cp: 0.1 4c82b29
// COCI 2020 Anagramistica: choose some of the n words so that exactly k pairs
// of chosen words are anagrams of each other. Count the choices.
const P = 1000000007
type Z = Mod<P>
let n: 1..=2000 = read()
let k: 0..=2000 = read()
domain Letter = 'a'..='z'
domain Pairs = 0..=k

// words that are anagrams of each other form a class; only class sizes matter
var class_size: Map<[Letter]Int, Int> = []
for _ in 0..n:
    let word: [domain W]Char = read()
    var letters: [Letter]Int = 0
    for c in word: letters[Letter(c)] += 1
    class_size[letters] += 1

domain Size = 0..=n
var choose: [Size, Size]Z = Z(0)            // Pascal's triangle
for i in Size:
    choose[i, Size.first] = Z(1)
    for j in 1..=i.idx:
        choose[i, Size(j)] = choose[Size(i.idx - 1), Size(j - 1)] + choose[Size(i.idx - 1), Size(j)]

// ways[p]: choices among the classes seen so far that make exactly p pairs
var ways: [Pairs]Z = Z(0)
ways[Pairs.first] = Z(1)
for (_, size) in class_size:
    var next: [Pairs]Z = Z(0)
    for p in Pairs:
        for take in 0..=size:
            let made = p.idx + take * (take - 1) / 2
            if made > k: break
            next[Pairs(made)] += choose[Size(size), Size(take)] * ways[p]
    ways = next
print(ways[Pairs.last])

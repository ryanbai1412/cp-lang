// Source: EGOI 2023 Day 2 Problem 4 (Guessing Game), https://dmoj.ca/problem/egoi23p8
// Problem: two runs, N = 1e5 houses. Run 1 prints K, then labels each of N - 1 houses, given in
//   an adversarial order, with 1..=K; Emma labels the last house (hers) arbitrarily. Run 2 reads
//   all N labels and prints two guesses, one of which must be her house. Smaller K scores more.
// Solution: K = 16. The first N - 13 houses get 1, with index sum s mod N; s / 2 is sent as a
//   composition of 11 into 9 parts (C(19, 8) >= N / 2) on the next 11 houses, labels 2..=10. The
//   last one gets 11 + j / 2, j the slot of the unseen house among the other 12 non-1 houses.
//   Run 2: if Emma wrote 1, her house is (sum of 1-houses - s) mod N for both parities of s; if
//   she wrote 11..=16, guess both such houses; else the flag names her pair of slots. O(N).
// original: corpus/submissions/5659039.cpp (91/100)
// Bounds: phase: 1..=2; n: 13..=150_000; h, flag: 0..n; label: 1..=16

const PARTS = 9
const TOTAL = 11

// every way to write `total` as `parts` ordered non-negative digits, in lexicographic order
fn compositions(parts: Int, total: Int) -> List<List<Int>>:
    if parts == 1: return [[total]]
    var all: List<List<Int>> = []
    for d in 0..=total:
        for rest in compositions(parts - 1, total - d): all.push([d] + rest)
    all
let codes = compositions(PARTS, TOTAL)

let phase: 1..=2 = read()
let n: 13..=150_000 = read()
domain H = 0..n

if phase == 1:
    print(16)
    var seen: [H]Bool = false
    var one_sum = 0
    for _ in 0..n - 13:
        let h: H = read()
        seen[h] = true
        one_sum += h.ord
        print(1)
    let rest = H.filter(h => not seen[h]).collect()  // the 12 coded houses and Emma's
    for (i, d) in codes[one_sum % n / 2].items:
        for _ in 0..d:
            let h: H = read()
            seen[h] = true
            print(2 + i)
    let flag: H = read()
    seen[flag] = true
    let others = rest.filter(h => h != flag).collect()
    print(11 + (0..others.len).find(j => not seen[others[j]])! / 2)
else:
    let label: [H]1..=16 = read()
    let ones = H.filter(h => label[h] == 1).collect()
    let coded = H.filter(h => 2 <= label[h] <= 10).collect()
    let flags = H.filter(h => label[h] >= 11).collect()
    if ones.len == n - 12:
        // Emma wrote 1: the composition is whole, and her index is the 1-houses' sum less s
        let count = [coded.count(h => label[h] == 2 + i) for i in 0..PARTS]
        let at = (0..codes.len).find(j => codes[j] == count)!
        let known = sum(h.ord for h in ones)
        print((known - 2 * at) % n, (known - 2 * at - 1) % n)
    elif flags.len == 2: print(flags[0], flags[1])
    else:
        let pair = (label[flags[0]] - 11) * 2
        print(coded[pair], coded[pair + 1])

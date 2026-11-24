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
// Bounds: phase: 1..=2; n: 13..=150_000
const PARTS = 9
const TOTAL = 11

// every way to write TOTAL as PARTS ordered non-negative digits, packed
// base 12, in increasing order
var codes: List<Int> = []
var rank = Map<Int, Int>()
fn gen(slots: Int, left: Int, code: Int):
    if slots == 0:
        if left == 0:
            rank[code] = codes.len
            codes.push(code)
        return
    for d in 0..=left: gen(slots - 1, left - d, code * 12 + d)
gen(PARTS, TOTAL, 0)

fn digit(code: Int, i: Int) -> Int:
    var c = code
    for _ in i + 1..PARTS: c /= 12
    return c % 12

let phase: 1..=2 = read()
let n: 13..=150_000 = read()
domain H = 0..n

if phase == 1:
    var seen: [H]Bool = false
    print(16)
    var one_sum = 0
    for _ in 0..n - 13:
        let h: H = read()
        seen[h] = true
        one_sum = (one_sum + h.idx) % n
        print(1)
    var rest: List<H> = []
    for h in H:
        if not seen[h]: rest.push(h)
    let code = codes[one_sum / 2]
    for i in 0..PARTS:
        for _ in 0..digit(code, i):
            let h: H = read()
            seen[h] = true
            print(2 + i)
    // the last label says which pair of the 12 remaining unlabelled-by-1
    // houses contains the one that never shows up
    let flag: H = read()
    seen[flag] = true
    var slot = 0
    var k = 0
    for h in rest:
        if h != flag:
            if not seen[h]: slot = k
            k += 1
    print(11 + slot / 2)
else:
    let label: [H]0..=16 = read()
    var ones: List<Int> = []
    var coded: List<Int> = []
    var flags: List<Int> = []
    var count: [0..PARTS]Int = 0
    for h in H:
        if label[h] == 1: ones.push(h.idx)
        elif label[h] >= 11: flags.push(h.idx)
        elif label[h] >= 2:
            coded.push(h.idx)
            count[label[h] - 2] += 1
    if ones.len != n - 13:
        // a 1-house was erased: recover the index sum and solve for it
        var code = 0
        for i in 0..PARTS: code = code * 12 + count[i]
        let at = rank.get(code) ?? codes.len
        var known = 0
        for h in ones: known += h
        print((known - 2 * at) % n, (known - 2 * at - 1) % n)
    elif flags.len == 2: print(flags[0], flags[1])
    else:
        let pair = (label[H(flags[0])] - 11) * 2
        print(coded[pair], coded[pair + 1])

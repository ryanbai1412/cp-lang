// Source: DMOPC '21 Contest 2 P3 (Divisions), https://dmoj.ca/problem/dmopc21c2p3
// Problem: interactive. Find N <= 1000 hidden numbers 0 <= F_i < 2^60 in order with at most
//   32000 guesses in all: guessing G for F_i answers the fewest halvings (rounding down) of
//   F_i or G that make them equal; 0 means correct, go on to the next.
// Solution: the answer is the tree distance with parent(x) = x >> 1. Guess 0 to learn the
//   depth (bit length) of F_i, then guesses of that depth: the known prefix and random lower
//   bits. The answer is twice the height of the first wrong bit, so that bit gets fixed and
//   about two bits are learnt per guess: some 31 guesses per number.
// original: corpus/submissions/3946741.cpp
// Runs with an inline local judge because `proto/cpi` buffers all stdin
// tokens upfront. The judge answers a guess with the distance between the
// guess and the hidden node in the tree where the parent of x is x >> 1.

let n: Int = read()
domain T = 0..n
let hidden: [T]Int = read()

var queries = 0
var found: List<Int> = []

fn ask(target: Int, guess: Int) -> Int:
    queries += 1
    assert 0 <= guess < (1 << 60)
    var (a, b, d) = (target, guess, 0)
    while a != b:
        if a > b: a = a >> 1
        else: b = b >> 1
        d += 1
    d

for i in T:
    let depth = ask(hidden[i], 0)
    var cur = 0
    if depth > 0:
        // `cur` holds the known prefix: every bit from `low` up is right.
        var low = depth - 1
        cur = 1 << low
        while true:
            let guess = cur | rand(0, (1 << low) - 1)
            let d = ask(hidden[i], guess)
            if d == 0:
                cur = guess
                break
            // Same depth, so d is twice the height of the first wrong bit.
            low = d / 2 - 1
            cur = (cur | ((guess >> low) << low)) ^ (1 << low)
    found.push(cur)

for i in T: assert found[i.ord] == hidden[i]
assert queries < 32000
for x in found: print(x)

// Source: DMOPC '21 Contest 2 P3 (Divisions), https://dmoj.ca/problem/dmopc21c2p3
// Problem: interactive. Find N <= 1000 hidden numbers 0 <= F_i < 2^60 in order with at most
//   32000 guesses in all: guessing G for F_i answers the fewest halvings (rounding down) of
//   F_i or G that make them equal; 0 means correct, go on to the next.
// Solution: the answer is the tree distance with parent(x) = x >> 1. Guess 0 to learn the
//   depth (bit length) of F_i, then guesses of that depth: the known prefix and random lower
//   bits. The answer is twice the height of the first wrong bit, so that bit gets fixed and
//   about two bits are learnt per guess: some 31 guesses per number.
// original: corpus/submissions/3946741.cpp
let wire = Wire()
@!multitest
let depth: 0..=60 = wire.ask("0")
if depth > 0:
    // every bit from `low` up is known in `known`
    var low = depth - 1
    var known = 1 << low
    loop:
        let guess = known | rand(0, (1 << low) - 1)
        let missed: 0..=120 = wire.ask("{guess}")
        if missed == 0: break
        // same depth, so `missed` is twice the height of the first wrong bit
        low = missed / 2 - 1
        known = (known | ((guess >> low) << low)) ^ (1 << low)

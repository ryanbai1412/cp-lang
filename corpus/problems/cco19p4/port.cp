// Source: CCO 2019 P4 (Card Scoring), https://dmoj.ca/problem/cco19p4
// limits: 2750 ms, 1024 MB
// Problem: n <= 1e6 cards with values in 1..=n, drawn in order; keep or discard each, the
//   hand holds one value only, and scoring a hand of x cards gives x^(k/2), 2 <= k <= 4.
//   Print the maximum total score (relative error 1e-6).
// Solution: dp after card i = max over later cards j of the same value of
//   score(#value in i..=j) + dp after j. Going right to left, each value keeps a std `LiChao`
//   over how many of its cards are still ahead, holding one curve score(m - x) + dp per card
//   (score is convex, so two curves cross once). O(n log n).
// original: corpus/submissions/3617807.cpp
// Bounds: k: 2..=4; n: 1..=1e6; a: 1..=n
let (k, n): (2..=4, 1..=1e6) = read()
domain Card = 0..n
domain Value = 1..=n
let a: [Card]Value = read()

// the score of a hand of x cards, for every x
let score = [if k == 2: F64(x) elif k == 3: F64(x) * sqrt(F64(x)) else: F64(x * x) for x in 0..=n + 1]

// keeping cards of one value from here up to the one with m cards left ahead, then dp after it
struct Run(m: Int, after: F64):
    fn at(self, x: Int) -> F64 = score[max(m - x, 0)] + after

var ahead: [Value]Int = 0  // cards of each value not passed yet
for v in a: ahead[v] += 1
var runs: [Value]LiChao<Run, F64> = [Value] v => LiChao<Run, F64>(max, over: 0..ahead[v] + 1)

var best = 0.0  // the best score from the current card on
for v in a.rev:
    ahead[v] -= 1
    runs[v].add(Run(ahead[v] + 1, best))
    best = runs[v].at(ahead[v])!
print(best)

// Source: AtCoder Educational DP Contest I (Coins), https://atcoder.jp/contests/dp/tasks/dp_i
// Problem: N coins, N odd and <= 2999; coin i shows heads with probability p_i, 0 < p_i < 1.
//   Print the probability of more heads than tails, to absolute error 1e-9.
// Solution: chance[h] is the probability of h heads among the coins tossed so far. Each coin
//   moves chance[h] * p up to h + 1 and keeps chance[h] * (1 - p); going down over h updates
//   in place. Sum chance[h] over 2h > N. O(N^2).
// original: corpus/submissions/4131648.cpp
// Bounds: n: 1..=2999
let n: 1..=2999 = read()
let heads: [n]F64 = read()

domain Heads = 0..=n
var chance: [Heads]F64 = 0.0
chance[Heads.first!] = 1.0
for p in heads:
    for h in Heads.rev:
        if let up = h.next: chance[up] += chance[h] * p
        chance[h] *= 1.0 - p
print(sum(chance[h] for h in Heads if 2 * h.label > n), precision: 10)

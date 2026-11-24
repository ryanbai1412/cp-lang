// Source: CEOI 2017 P2 (Sure Bet), https://dmoj.ca/problem/ceoi17p2
// limits: 1000 ms, 128 MB
// Problem: n <= 1e5 bookmakers offer odds a_i on outcome 1 and b_i on outcome 2 (real,
//   1..1000). Each bet costs 1 and pays its odds if it wins; bet on any subset of the 2n
//   offers. Print the largest profit guaranteed whatever happens, to 4 decimals.
// Solution: with i bets on outcome 1 and j on outcome 2 take the largest odds, so sort
//   both descending. Greedy: the next bet goes on the outcome that would currently pay
//   less; track max of min(pays_a, pays_b) - (i + j) over all steps. O(n log n).
// original: corpus/submissions/5324109.cpp
// Bounds: n: 1..=1e5
let n: 1..=1e5 = read()
let odds: [n](a: F64, b: F64) = read()
let a = odds.map(o => o.a).sorted(by: desc)
let b = odds.map(o => o.b).sorted(by: desc)

var (i, j, pays_a, pays_b, sure) = (0, 0, 0.0, 0.0, 0.0)
while i < n or j < n:
    if j == n or (i < n and pays_a <= pays_b):
        pays_a += a[i]
        i += 1
    else:
        pays_b += b[j]
        j += 1
    sure max= min(pays_a, pays_b) - (i + j)
print(sure, precision: 4)

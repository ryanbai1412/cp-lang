// Source: CEOI 2017 P2 (Sure Bet), https://dmoj.ca/problem/ceoi17p2
// Problem: n <= 1e5 bookmakers offer odds a_i on outcome 1 and b_i on outcome 2 (real,
//   1..1000). Each bet costs 1 and pays its odds if it wins; bet on any subset of the 2n
//   offers. Print the largest profit guaranteed whatever happens, to 4 decimals.
// Solution: with i bets on outcome 1 and j on outcome 2 take the largest odds, so sort
//   both descending. Greedy: the next bet goes on the outcome that would currently pay
//   less; track max of min(pays_a, pays_b) - (i + j) over all steps. O(n log n).
// original: corpus/submissions/5324109.cpp
let n: 1..=1e5 = read()
let odds: [n](a: F64, b: F64) = read()  // WISH R9: 1.0..=1000.0
let a = [o.a for o in odds].sorted(by: x => desc(x))
let b = [o.b for o in odds].sorted(by: x => desc(x))

// Only the best odds of each outcome are worth a bet, and the next bet
// always goes on the outcome that would pay less.
var (i, j, pays_a, pays_b, sure) = (0, 0, 0.0, 0.0, 0.0)
while i < n or j < n:  // WISH R8: two lists, two hand-moved places
    if j == n or (i < n and pays_a <= pays_b):
        pays_a += a[i]
        i += 1
    else:
        pays_b += b[j]
        j += 1
    sure max= min(pays_a, pays_b) - (i + j)
print(sure, precision: 4)

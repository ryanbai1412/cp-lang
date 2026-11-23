// CEOI 2017 Sure Bet: n bookmakers each offer odds a on one outcome and b on
// the other. A bet costs 1 and pays its odds if it wins. Place any bets to
// make the profit you are sure of, whichever outcome happens, as large as
// possible.
// original: corpus/submissions/5324109.cpp
let n: 1..=1e5 = read()
let odds: [n](a: F64, b: F64) = read()  // WISH R9: 1.0..=1000.0
let a = [o.a for o in odds].sorted(desc: true)
let b = [o.b for o in odds].sorted(desc: true)

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

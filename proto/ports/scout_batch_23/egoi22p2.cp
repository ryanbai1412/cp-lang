// EGOI '22 P2 - Lego Wall (DMOJ: egoi22p2, 15pt, TL 3.0s, ML 256MB)
// Statement-first CP Lang solution (brick sizes 1x1 and 1x2 and w * h <= 250000
// verified against corpus/submissions/5199888.cpp; initial recall used 1x2/1x3).
//
// Count solid (no full-height vertical seam) walls of width w and height h made from
// 1x1 and 1x2 horizontal bricks, modulo 10^9 + 7, with w * h <= 250000.
//
// Row tilings: F[0] = 1, F[1] = 1, F[i] = F[i - 1] + F[i - 2] (Fibonacci).
// All walls of width i and height h: T[i] = F[i]^h.
// Solid walls S[i] satisfy the formal power series identity:
//   T(x) = 1 / (1 - S(x))   <=>   S(x) = 1 - T(x)^(-1)
// i.e. S[i] = T[i] - sum_{j = 1..i-1} S[j] * T[i - j]  (mod 10^9 + 7).

type Z = Mod<1000000007>

// WISH MG11: product constraint `w * h <= 250000` is non-typeable as independent ranges
let (w, h): (1..=250000, 1..=250000) = read()
assert w * h <= 250000

var row = List.filled(w + 1, Z(0))
row[0] = Z(1)
row[1] = Z(1)
for i in 2..=w:
    row[i] = row[i - 1] + row[i - 2]

var total = List.filled(w + 1, Z(0))
total[0] = Z(1)
for i in 1..=w:
    total[i] = row[i].pow(h)

// WISH MG6: Formal power series `Poly<Z>.inv(w + 1)` in std.poly (using NTT for
// NTT-friendly primes or 3-prime/split-base MTT for 10^9 + 7) solves BOTH small-w
// and large-w regimes in one O(w log w) call, avoiding the corpus C++'s split into
// an O(w^2) convolution for w <= 10^4 and a separate O(w * h^2) column-profile DP:
//   let solid = Poly<Z>(1) - Poly<Z>(total).inv(w + 1)
//   print(solid[w])
var solid = List.filled(w + 1, Z(0))
for i in 1..=w:
    var cur = total[i]
    for j in 1..i:
        cur -= solid[j] * total[i - j]
    solid[i] = cur

print(solid[w])

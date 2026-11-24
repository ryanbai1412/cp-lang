// Source: EGOI 2022 Day 1 (Lego Wall), https://dmoj.ca/problem/egoi22p2
// Problem: count the walls of width w and height h (w * h <= 5e5) built from 1x1 and 2x1
//   bricks with no holes that are connected (bricks touching one above the other link),
//   mod 1e9+7.
// Solution: a row of width i has fib(i) tilings, so total[i] = fib(i)^h walls of width i.
//   A wall is disconnected exactly when some vertical line cuts every row; split off the
//   connected part before the first such line: solid[i] = total[i] - sum over j < i of
//   solid[j] * total[i - j]. O(w^2).
// Note: runnable on proto/cp.py lib-run today.

type Z = Mod<1000000007>

let (w, h): (Int, Int) = read()

var row = List.filled(w + 1, Z.zero)
row[0] = 1
row[1] = 1
for i in 2..=w:
    row[i] = row[i - 1] + row[i - 2]

var total = List.filled(w + 1, Z.zero)
total[0] = 1
for i in 1..=w:
    total[i] = row[i] ** h

var solid = List.filled(w + 1, Z.zero)
for i in 1..=w:
    var cur = total[i]
    for j in 1..i:
        cur -= solid[j] * total[i - j]
    solid[i] = cur

print(solid[w])

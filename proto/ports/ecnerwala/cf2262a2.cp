// Codeforces 2262 A2: count subsets of 0..n that avoid every bucket
// [a[i] * i, (a[i] + 1) * i) yet hit each lower bucket [j * i, (j + 1) * i),
// j < a[i].  `need[lo]` is the smallest right end of a must-hit interval
// starting at or after lo; ways[i] counts valid choices whose smallest chosen
// point is i, so the next chosen point must be below need[i].
use std.prefix.{prefix1}

const MOD = 1e9 + 7
type Z = Mod<MOD>

@!multitest
let n: 1..=2e5 = read()
let a: [1..=n]Int = read()
var diff: [0..=n]Int = 0
var need: [0..=n]Int = n + 1
for i in 1..=n:
    for j in 0..a[i]: need[j * i] min= min(n, j * i + i)
    diff[min(n, a[i] * i)] += 1
    diff[min(n, a[i] * i + i)] -= 1
let cover = prefix1(diff)
for i in (0..n).rev: need[i] min= need[i + 1]

var ways: [0..=n]Z = 0
ways[n] = 1
var tot: Z = 1
var hi = n + 1
for i in (0..n).rev:
    if cover.prefix(i + 1) == 0:
        ways[i] = tot
        tot += ways[i]
    while hi > need[i]:
        hi -= 1
        tot -= ways[hi]
print(tot)

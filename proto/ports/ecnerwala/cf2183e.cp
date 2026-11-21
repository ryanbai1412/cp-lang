// Codeforces 2183 E: LCM is Legendary Counting Master.
// 1/lcm(x, y) <= 1/x - 1/y for x < y, so the sum telescopes to at most 1/a_1:
// a good sequence starts at 1 and steps x -> x + d with d dividing x.
type Z = Mod<998_244_353>

@!multitest
let (n, m): (2..=3000, 2..=3000) = read()
let a: [0..n]0..=m = read()
var ways: [1..=m]Z = 0
var next: [1..=m]Z = 0
if a[0] in [0, 1]: ways[1] = 1
for i in 1..n:
    next.fill(0)
    for d in 1..=m:
        for j in 1..m / d: next[(j + 1) * d] += ways[j * d]
    if a[i] != 0:
        for x in 1..=m if x != a[i]: next[x] = 0
    swap(mut ways, mut next)
print(ways.sum())

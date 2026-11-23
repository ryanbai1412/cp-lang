// Codeforces 2183 E: LCM is Legendary Counting Master (dream).
// 1/lcm(x, y) <= 1/x - 1/y, so the sum telescopes to at most 1/a_1:
// a good sequence starts at 1 and steps x -> x + d with d dividing x.
type Z = Mod<998_244_353>

@!multitest
let (n, m): (2..=3000, 2..=3000) = read()
let a: [0..n]?(1..=m) = read(None: 0)
var ways, next: [1..=m]Z = 0
if a[0] in [None, 1]: ways[1] = 1
for i in 1..n:
    for d in 1..=m:
        for x in (d..=m - d).step(d): next[x + d] += ways[x]
    if let v = a[i]:
        for x in 1..=m if x != v: next[x] = 0
    ways = next.take_all()
print(ways.sum())

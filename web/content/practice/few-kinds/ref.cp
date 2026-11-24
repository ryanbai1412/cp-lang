let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=1e9 = read()
var have: Counter<Int, Int> = Counter()
var distinct = 0
var lo = 0
var best = 0
for (hi, x) in a.items:
    if have.get(x) == 0: distinct += 1
    have[x] += 1
    while distinct > k:
        let y = a[lo]
        have[y] -= 1
        if have.get(y) == 0: distinct -= 1
        lo += 1
    best max= hi - lo + 1
print(best)

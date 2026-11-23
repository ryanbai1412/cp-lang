// CF 2075D Equalization. Shifting x right by a and y by b with distinct ks is a 0/1
// knapsack over k = 1..=K: best[a, b] is the cheapest set of ks split into sums a and b.
const K = 60  // 1e17 < 2^57, so a shift past 57 already gives 0

var best: [0..=K, 0..=K]?Int = None
best[0, 0] = 0
for k in 1..=K:
    for a in (0..=K).rev, b in (0..=K).rev:
        if a >= k: best[a, b] min= best[a - k, b] + (1 << k)
        if b >= k: best[a, b] min= best[a, b - k] + (1 << k)

@!multitest
let x, y: 0..=1e17 = read()
var ans: ?Int = None
for a in 0..=K, b in 0..=K if x >> a == y >> b:
    ans min= best[a, b]
print(ans!)

// CF 2075D Equalization, revised: three numbers x, y, z (t <= 1000) must all become equal.
// Same 0/1 knapsack over k = 1..=K, now splitting the ks into three shift sums.
const K = 60  // 1e17 < 2^57, so a shift past 57 already gives 0

var best: [0..=K, 0..=K, 0..=K]?Int = None
best[0, 0, 0] = 0
for k in 1..=K:
    for a in (0..=K).rev, b in (0..=K).rev, c in (0..=K).rev:
        if a >= k: best[a, b, c] min= best[a - k, b, c] + (1 << k)
        if b >= k: best[a, b, c] min= best[a, b - k, c] + (1 << k)
        if c >= k: best[a, b, c] min= best[a, b, c - k] + (1 << k)

@!multitest
let x, y, z: 0..=1e17 = read()
var ans: ?Int = None
for a in 0..=K, b in 0..=K, c in 0..=K if x >> a == y >> b and y >> b == z >> c:
    ans min= best[a, b, c]
print(ans!)

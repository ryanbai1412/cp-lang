// AtCoder DP M Candies: share all k candies among n children, child i getting
// at most limit[i]. Giving a child 0..=cap candies turns ways[c] into the sum
// of a window of the previous row, slid along as c grows.
type Z = Mod<1_000_000_007>
let (n, k): (1..=100, 0..=1e5) = read()
let limit: [0..n]0..=k = read()

var ways: [0..=k]Z = 0
ways[0] = 1
for cap in limit:
    var next: [0..=k]Z = 0
    var window: Z = 0
    for c in 0..=k:
        window += ways[c]
        if c > cap: window -= ways[c - cap - 1]
        next[c] = window
    ways = next
print(ways[k])

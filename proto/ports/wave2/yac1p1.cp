// Yet Another Contest 1 P1: pair up a's with b's, then add 1 to any element k
// times, to maximise the sum of products. Pair in sorted order and spend every
// increment on the largest pair, keeping its two sides as level as possible.
type Z = Mod<1_000_000_007>
let (n, k): (1..=2e5, 0..=1e9) = read()
let a: [0..n]1..=1e9 = read()
let b: [0..n]1..=1e9 = read()
let (lo_a, lo_b) = (sorted(a), sorted(b))

var (x, y) = (lo_a[n - 1], lo_b[n - 1])
let level = min(k, abs(x - y))          // first bring the smaller side up
if x < y: x += level else: y += level
x += (k - level) / 2                    // then split what is left
y += (k - level + 1) / 2
var total = Z(x) * y
for i in 0..n - 1: total += Z(lo_a[i]) * lo_b[i]
print(total)

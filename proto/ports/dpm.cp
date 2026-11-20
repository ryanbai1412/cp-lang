// cp: 0.1 1f59ac0+dirty
// AtCoder DP M Candies: share k candies among n children, child i getting at
// most a[i]. ways[c] after a child is a window sum of the previous row, read
// off its prefix sums.
const P = 1000000007
type Z = Mod<P>
let n: 1..=100 = read()
let k: 0..=1e5 = read()
domain C = 0..=k
let limit: [n]0..=1e5 = read()

var ways: [C]Z = Z(0)
ways[C.first] = Z(1)
for cap in limit:
    var pre: [C.bounds]Z = Z(0)
    for c in C: pre[c.r] = pre[c.l] + ways[c]
    for c in C: ways[c] = pre[c.r] - pre[(c - cap ?? C.first).l]
print(ways[C.last])

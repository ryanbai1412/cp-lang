// Yet Another Contest 1 P1: pair up a's with b's and add 1 to any element k
// times to maximise the sum of products. Pair in sorted order and spend every
// increment on the largest pair, keeping its two sides as level as possible.
const P = 1000000007
type Z = Mod<P>
let n: 1..=2e5 = read()
let k: 0..=1e9 = read()
let a_in: [n]1..=1e9 = read()
let b_in: [n]1..=1e9 = read()
let a = a_in.sorted()
let b = b_in.sorted()

var x = a.last!
var y = b.last!
let level = min(k, abs(x - y))          // first bring the smaller side up
if x < y: x += level else: y += level
x += (k - level) / 2                    // then split what is left
y += (k - level + 1) / 2

print(sum(Z(a[i]) * Z(b[i]) for i in 0..n - 1) + Z(x) * Z(y))

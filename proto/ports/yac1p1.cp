// cp: 0.1 0268fdb+dirty
// Yet Another Contest 1 P1: pair up a's with b's and add 1 to any element k
// times to maximise the sum of products. Pair in sorted order and spend every
// increment on the largest pair, keeping its two sides as level as possible.
const P = 1000000007
type Z = Mod<P>
let n: 1..=2e5 = read()
let k: 0..=1e9 = read()
domain Rank = 1..=n
let a_in: [Rank]1..=1e9 = read()
let b_in: [Rank]1..=1e9 = read()
let a = sorted(a_in)
let b = sorted(b_in)

var x = a[Rank.last]
var y = b[Rank.last]
let level = min(k, abs(x - y))          // first bring the smaller side up
if x < y: x += level else: y += level
x += (k - level) / 2                    // then split what is left
y += (k - level + 1) / 2

print(sum(Z(a[i]) * Z(b[i]) for i in Rank if i != Rank.last) + Z(x) * Z(y))

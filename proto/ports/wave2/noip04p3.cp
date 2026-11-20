// NOIP 2004 Chorus Formation: remove the fewest singers so that heights
// strictly rise to some singer and strictly fall after: the longest rising
// run ending at each singer plus the longest falling run starting there.
let n: 2..=100 = read()
let h: [0..n]130..=230 = read()

var up: [0..n]Int = 1
var down: [0..n]Int = 1
for i in 0..n: up[i] += max(up[j] for j in 0..i if h[j] < h[i]) ?? 0
for i in (0..n).rev: down[i] += max(down[j] for j in i + 1..n if h[j] < h[i]) ?? 0
print(n - max(up[i] + down[i] - 1 for i in 0..n))

// NOIP 2004 Chorus Formation: remove the fewest singers so that heights
// strictly rise to some singer and strictly fall after: longest rising run
// ending at each singer plus longest falling run starting there.
let n: 2..=100 = read()
domain I = 1..=n
let h: [I]130..=230 = read()

var up: [I]Int = 1
var down: [I]Int = 1
for i in I:
    up[i] = 1 + (max(up[j] for j in I if j < i and h[j] < h[i]) ?? 0)
for i in I.rev:
    down[i] = 1 + (max(down[j] for j in I if j > i and h[j] < h[i]) ?? 0)
print(n - max(up[i] + down[i] - 1 for i in I))

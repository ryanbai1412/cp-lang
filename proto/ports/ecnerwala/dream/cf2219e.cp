// Codeforces 2219 E: Weird Chessboard.
// Every cell has even attacking-piece parity iff the 2D prefix XOR equals a[i][j],
// equivalently a[i][j] = a[i][j + 1] ^ a[i + 1][j] with top and left borders zero
// except at (0, n - 1) and (n - 1, 0). Placing the period-3 pattern (i - j + 1) % 3 != 0
// on [r0..n) x [c0..n) (r0 = (n - 1) / 2, c0 = n - 1 - r0) and propagating the
// recurrence into the two lower-right triangles keeps all cells above the anti-diagonal
// zero and achieves >= (1/6 + 2/15) n^2 = 0.3 n^2 pieces.

@!multitest
let n: 1..=5000 = read()
domain R = 0..n
var a: [R][R]0..=1 = 0

fn mirror(i: R) -> R = R.nth(n - 1 - i.ord)
let r0 = R.nth((n - 1) / 2)
let c0 = mirror(r0)

for i in r0..=R.last:
    for j in c0..=R.last:
        a[i][j] = Int((i.ord - j.ord + 1) % 3 != 0)

for i in (R.first..r0).rev:
    for j in mirror(i)..=R.last:
        a[i][j] = a[i + 1][j - 1] ^ a[i][j - 1]

for j in (R.first..c0).rev:
    for i in mirror(j)..=R.last:
        a[i][j] = a[i - 1][j + 1] ^ a[i - 1][j]

for i in R:
    print(a[i])

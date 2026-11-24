// Source: Codeforces 2180D, Insolvable Disks, https://codeforces.com/contest/2180/problem/D
// Problem: multitest, sum n <= 2e6; centres 0 <= x_1 < ... < x_n <= 1e9 on a line. Choose a
//   positive radius per disk, no two overlapping, to maximize the number of tangent pairs.
// Solution: only neighbours can touch. Sweep left to right keeping the open interval of radii
//   for disk i - 1 in the current tangent chain: across gap d it is clamped below d and
//   reflected to d - r, counting a tangent pair. If it empties, start over with disk i free in
//   (0, d) and no new pair. O(n).

@!multitest
let n: 1..=2_000_000 = read()
let x: [0..n]0..=10 ** 9 = read()
if n == 1:
    print(0)
    return

var v = 1
var (r_mi, r_ma) = (0, x[1] - x[0])
for i in 2..n:
    let d = x[i] - x[i - 1]
    r_ma min= d
    if r_ma <= r_mi:
        (r_mi, r_ma) = (0, d)
    else:
        v += 1
        (r_mi, r_ma) = (d - r_ma, d - r_mi)
print(v)

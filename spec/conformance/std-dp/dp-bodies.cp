// expect: ok, output "1 3\n3 5 6 0\n0 1 2 3 4 5 6 7\n0 0\n0 1\n1 1 2 1 3\n1 2 3\n1 2 2\nYES"
// copies: 1
use std.dp.{supermasks, masks_of_size, subset_folds, lis_ends}
use std.rand.{Rng}
print(supermasks(0b01, 2).collect())
print(masks_of_size(3, 2).collect(), masks_of_size(2, 3).collect(), masks_of_size(4, 0).collect())
print(subset_folds([1, 2, 4], +))
print(smawk(2, 3, (i, j) => (j - i) * (j - i)))
domain P = 0..5
let xs: [P]Int = [3, 1, 4, 1, 5]
print(lis_ends(xs))
domain Q = 0..3
let ys: [Q]Int = [1, 1, 2]
print(lis_ends(ys, strict: false))
let zs: [Q]Int = [3, 1, 2]
print(lis_ends(zs, by: x => -x))
var rng = Rng(5)
var ok = true
for _ in 0..200:
    let rows = rng.int(1, 12)
    let cols = rng.int(1, 12)
    let a = [rng.int(-20, 20) for _ in 0..rows]
    let b = [rng.int(-20, 20) for _ in 0..cols]
    let f = (i: Int, j: Int) => (j * 3 - i * 2) * (j * 3 - i * 2) + b[j]
    let got = smawk(rows, cols, f)
    for i in 0..rows:
        var bv = f(i, 0)
        var bj = 0
        for j in 1..cols:
            if f(i, j) < bv:
                bv = f(i, j)
                bj = j
        if got[i].val != bv or got[i].at != bj: ok = false
print(yes(ok))

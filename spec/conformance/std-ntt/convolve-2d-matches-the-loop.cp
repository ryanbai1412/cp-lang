// expect: ok, output "YES\n1 1\n2 2\n0"
// `convolve_2d` is the 2D product of ragged row lists (a missing entry is zero), equal to the
// quadruple loop, in both engines; an input with no entries gives `[]`.
use std.ntt.{convolve_2d}
type Z = Mod<998_244_353>
let a: List<List<Z>> = [[1, 2, 3], [4], [], [5, 6]]
let b: List<List<Z>> = [[7, 8], [9, 10, 11]]
let got = convolve_2d(a, b)
var want = [[Z(0) for _ in 0..5] for _ in 0..5]
for (i, r) in a.items:
    for (j, x) in r.items:
        for (k, s) in b.items:
            for (l, y) in s.items: want[i + k][j + l] += x * y
print(yes(got == want))
let small = convolve_2d([[Z(1), Z(1)]], [[Z(1)], [Z(2)]])
for r in small: print(r)
let e: List<List<Z>> = [[]]
print(convolve_2d(a, e).len)

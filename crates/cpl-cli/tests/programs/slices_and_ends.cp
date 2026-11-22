// Syntax batch 1: slices, swaps, ranges open at either end, and the
// first and last of an array over domains.
var xs = [5, 4, 3, 2, 1]
print(xs[..2])
print(xs[1..3])
print(xs[2..])
print(xs[1..=3])
(xs[0], xs[4]) = (xs[4], xs[0])
print(xs)
var x = 1
var y = 2
(x, y) = (y, x)
print(x, y)

let n = 5
domain I = 0..n
let a: [I]Int = [I] i => i.to_idx + 10
let l = I(2)
for r in l..: print(a[r])
for r in ..l: print(a[r])
for i in 3..:
    if i > 5: break
    print(i)
print(a.first!, a.last!)

domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => r.to_idx * 3 + c.to_idx
print(g.first!, g.last!)
var ends: [I.cuts]Int = 0
ends[I.last!.r] = 9
print(ends.first!, ends.last!)

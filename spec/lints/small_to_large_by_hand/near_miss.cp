let n: 1..=100000 = read()
var xs: List<Int> = [2 * x for x in 0..n]
var ys: List<Int> = (0..n / 2).collect()
if xs.len < ys.len:
    (xs, ys) = (ys, xs)
for i in 0..ys.len:
    xs[i] += ys[i]
print(xs)

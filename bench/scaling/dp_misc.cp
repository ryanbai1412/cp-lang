use std.dp.{knuth, aliens, subset_sums, lis_ends, supermasks, masks_of_size, subset_folds}
let n: Int = read()
let reps = 1 << 17
var s = 0
let xs = [i * 7919 % n for i in 0..n]
var lg = 0
while (1 << (lg + 1)) <= n: lg += 1
var rt = 0
while (rt + 1) * (rt + 1) <= n: rt += 1

let span: (Int, Int) -> Int = (i, j) => j - i // within: dp.knuth
for _ in 0..8:
    s += knuth(rt, span)[0][rt] // timed: dp.knuth [let: n = \sqrt n]
let pen: Int -> (val: Int, count: Int) = lam => (val: lam, count: if lam < 0: 2 else: 0) // within: dp.aliens
for i in 0..reps:
    s += aliens(1, pen, lambdas: -n..n) // timed: dp.aliens
let bits = [i % 5 + 1 for i in 0..lg]
for _ in 0..200:
    s += subset_sums(bits).len // timed: dp.subset_sums [let: n = log_2 n]
    s += subset_folds(bits, +).len // timed: dp.subset_folds [let: n = log_2 n]
    s += supermasks(0, lg).count() // timed: dp.supermasks [let: bits = log_2 n; m = 0]
for _ in 0..4000:
    s += masks_of_size(lg, 3).count() // timed: dp.masks_of_size [let: bits = log_2 n] [fixed: k]
for _ in 0..40:
    s += xs.lis_ends().len // timed: dp.lis_ends
print(s)

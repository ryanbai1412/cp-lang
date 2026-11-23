// 1915E Romantic Glasses
// in: t; per test n, then a_1..a_n. out: YES/NO per test
@!multitest
let n: Int = read()
let a: [n]Int = read()
let pre = [if i % 2 == 0: a[i] else: -a[i] for i in 0..n].scan(+)
print(yes(pre.sorted().adjacent.any((x, y) => x == y)))

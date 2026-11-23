// 1922C Closest Cities
// in: t; per test n, a_1..a_n (sorted), m, then m lines "x y". out: min cost per query
@!multitest
let n: Int = read()
let a: [n]Int = read()
let right = [if i == 0 or a[i + 1] - a[i] < a[i] - a[i - 1]: 1 else: a[i + 1] - a[i] for i in 0..n - 1].scan(+)
let left = [if i == n - 2 or a[i + 1] - a[i] < a[i + 2] - a[i + 1]: 1 else: a[i + 1] - a[i] for i in 0..n - 1].scan(+)
let m: Int = read()
repeat m:
    let x, y: Int = read()
    print(if x < y: right[y - 1] - right[x - 1] else: left[x - 1] - left[y - 1])

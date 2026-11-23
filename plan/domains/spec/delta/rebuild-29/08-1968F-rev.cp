// CF 1968F Equal XOR Segments, revised: instead of YES/NO, print where to cut: the end of
// the first part (2 parts) or the ends of the first two parts (3 parts), or -1.

@!multitest
let (n, q): (2..=2e5, 1..=2e5) = read()
let a = read<[n]0..2 ** 30>()
let p = a.scan(^)                      // p[k]: xor of the first k
let at = (0..=n).group(by: k => p[k])  // prefix value -> its cuts, increasing
repeat q:
    let (l, r): (1..=n, 1..=n) = read()
    let lo = l - 1
    if p[r] == p[lo]:
        print(l)
        continue
    let xs = at[p[r]]
    let i = xs[xs.partition_point(k => k > lo)]
    let ys = at[p[lo]]
    let j = ys[ys.partition_point(k => k >= r) - 1]
    if i < r and j > i: print(i, j) else: print(-1)

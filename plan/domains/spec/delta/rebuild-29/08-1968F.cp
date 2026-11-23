// CF 1968F Equal XOR Segments. With prefix xors p, the piece between cuts l-1 and r splits
// into 2 equal parts iff p[r] == p[l-1]; otherwise it needs 3 parts: the first cut i > l-1
// with p[i] == p[r], then a cut j with i < j < r and p[j] == p[l-1].

@!multitest
let (n, q): (2..=2e5, 1..=2e5) = read()
let a = read<[n]0..2 ** 30>()
let p = a.scan(^)                      // p[k]: xor of the first k
let at = (0..=n).group(by: k => p[k])  // prefix value -> its cuts, increasing
repeat q:
    let (l, r): (1..=n, 1..=n) = read()
    let lo = l - 1
    if p[r] == p[lo]:
        print("YES")
        continue
    let xs = at[p[r]]
    let i = xs[xs.partition_point(k => k > lo)]
    let ys = at[p[lo]]
    let j = ys[ys.partition_point(k => k >= r) - 1]
    print(yes(i < r and j > i))

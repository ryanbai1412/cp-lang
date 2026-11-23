// CF 1921F Sum of Progression: sqrt split on the step d. For d <= b keep two 2-D suffix
// tables over (d, i): plain sums along the progression and weighted sums; larger d is brute.
@!multitest
let (n, q) = read<(1..=1e5, 1..=2e5)>()
let a = read<[n]-1e8..=1e8>()
let b = n.isqrt
var suf: [b + 1, n + 1]Int = 0
var wsuf: [b + 1, n + 1]Int = 0
for d in 1..=b:
    for i in (0..n).rev:
        let nx = min(i + d, n)
        suf[d, i] = a[i] + suf[d, nx]
        wsuf[d, i] = wsuf[d, nx] + suf[d, i]

fn query(s: Int, d: Int, k: Int) -> Int:
    if d <= b:
        let e = min(s + d * k, n)
        return wsuf[d, s] - wsuf[d, e] - k * suf[d, e]
    var total = 0
    for i in 0..k:
        total += a[s + i * d] * (i + 1)
    total

let answers = [query(s - 1, d, k) for (s, d, k) in read<[q](1..=n, 1..=n, 1..=n)>()]
print(answers)

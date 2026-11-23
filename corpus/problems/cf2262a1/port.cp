// Codeforces 2262 A1 — Uncovered Floor-Division Buckets
//
// `a[i] = floor(x / i)` confines `x` to `[a[i] * i, (a[i] + 1) * i)`. With
// `n <= 200_000` and `a[i] <= 10^9`, `(a[i] + 1) * i <= 2e14` fits in 64-bit
// `Int`. Mark each bucket clamped to `0..=n` with a difference array and list
// all uncovered positions in `0..n`.

@!multitest
let n: 1..=200_000 = read()
let a: [1..=n]0..=10 ** 9 = read()
fn cap(x: Int) -> Int: min(n, x)

var diff: [0..=n]Int = 0
for i in 1..=n:
    diff[cap(a[i] * i)] += 1
    diff[cap((a[i] + 1) * i)] -= 1
let cover = scan(diff, +)
let free = [i for i in 0..n if cover.prefix(i + 1) == 0]
print(free.len)
print(free)

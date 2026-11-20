// Codeforces 2237 B: map each a[i] to its earliest legal slot via lower_bound
// in b, then greedily bubble the earliest element with slot <= i to position i.
@!multitest
let n: Int = read()
var a: [0..n]Int = read()
let b: [0..n]Int = read()
for i in 0..n: a[i] = bsearch_first(0..n, j => b[j] >= a[i]) ?? n

fn solve() -> Int:
    var ans = 0
    for i in 0..n:
        let j = first(j for j in i..n if a[j] <= i) ?? -1
        if j == -1: return -1
        let x = a[j]
        for k in (i + 1..=j).rev: a[k] = a[k - 1]
        a[i] = x
        ans += j - i
    ans

print(solve())

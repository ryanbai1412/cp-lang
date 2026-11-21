// Codeforces 2237 B: map each a[i] to its earliest legal slot via lower_bound
// in b, then greedily bubble the earliest element with slot <= i to position i.
@!multitest
let n: Int = read()
let a_in: [0..n]Int = read()
let b: [0..n]Int = read()
var a = [0..n] i => bsearch_first(0..n, j => b[j] >= a_in[i]) ?? n

var ans = 0
for i in 0..n:
    let j = first(j for j in i..n if a[j] <= i) else:
        print(-1)
        return
    let x = a[j]
    for k in (i + 1..=j).rev: a[k] = a[k - 1]
    a[i] = x
    ans += j - i
print(ans)

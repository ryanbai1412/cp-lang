// Codeforces 1904B: Collecting Game
// I/O: t tests; each n then a_1..a_n; print n numbers: for each i, how many other elements are removed starting from a_i.
@!multitest
let n: 1..=1e5 = read()
let a = read<[n]1..=1e9>()
let ord = a.keys.sorted(by: i => a[i])
let pre = ord.map(i => a[i]).scan(+)          // pre[k]: the k smallest values
var ans: [n]Int = 0
var far = n - 1                               // last sorted index reachable from k
for k in (0..n).rev:
    if k + 1 < n and pre[k + 1] < a[ord[k + 1]]: far = k
    ans[ord[k]] = far
print(ans)

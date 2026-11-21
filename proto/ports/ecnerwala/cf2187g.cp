// Codeforces 2187 G: find each node's furthest right child from the matrix S,
// then walk from index 0 following the unique valid parent link to assign 1..N.
@!multitest
let n: Int = read()
let s: [0..n]Str = read()
let ch1 = [0..n] i => first(j for j in (i + 1..n).rev if s[j][i] == '1')

var ans: [0..n]Int = 0
var cur = 0
for v in 1..=n:
    ans[cur] = v
    let pa0 = first(j for j in 0..cur if s[cur][j] == '1')
    let pa1 = first(j for j in (cur + 1..n).rev if s[cur][j] == '1')
    if let p0 = pa0, ch1[p0] == cur: cur = p0
    elif let p1 = pa1: cur = p1
print(ans)

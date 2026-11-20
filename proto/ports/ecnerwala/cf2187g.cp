// Codeforces 2187 G: find each node's nearest left/right children from the matrix S,
// then walk from index 0 following the unique valid parent link to assign 1..N.
@!multitest
let n: Int = read()
let s: [0..n]Str = read()

var ch0: [0..n]?Int = none
var ch1: [0..n]?Int = none
for i in 0..n:
    for j in (0..i).rev:
        if s[j][i] == '1': ch0[i] = j
    for j in i + 1..n:
        if s[j][i] == '1': ch1[i] = j

var ans: [0..n]Int = 0
var cur = 0
for v in 1..=n:
    ans[cur] = v
    var pa0: ?Int = none
    var pa1: ?Int = none
    for j in (0..cur).rev:
        if s[cur][j] == '1': pa0 = j
    for j in cur + 1..n:
        if s[cur][j] == '1': pa1 = j
    if let p0 = pa0 and ch1[p0] == some(cur):
        cur = p0
    elif let p1 = pa1:
        cur = p1
print(ans)

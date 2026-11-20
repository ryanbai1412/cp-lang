// Codeforces 2207 E1: validate the non-increasing suffix-MEX bounds, then
// reconstruct a valid array backwards using a stack of available values.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var good = true
for i in 0..n - 1:
    if a[i] < a[i + 1]: good = false
for i in 0..n:
    if a[i] < n - i - 1 or a[i] > n: good = false

if not good:
    print("NO")
else:
    var b: List<Int> = []
    var stk: List<Int> = []
    var prv = -1
    for i in (0..=n).rev:
        let cur = if i == 0: n else: a[i - 1]
        while prv < cur:
            prv += 1
            stk.push(prv)
        b.push(stk.pop()!)
    b.reverse()
    b.pop()
    print("YES")
    print(b)

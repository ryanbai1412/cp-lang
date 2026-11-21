// Codeforces 2207 E1: validate the non-increasing suffix-MEX bounds, then
// reconstruct a valid array backwards using a stack of available values.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()

if any(a[i] < a[i + 1] for i in 0..n - 1) or any(a[i] < n - i - 1 or a[i] > n for i in 0..n):
    print("NO")
    return

var b: [0..n]Int = 0
var stk: List<Int> = []
var prv = -1
for i in (0..=n).rev:
    let cur = if i == 0: n else: a[i - 1]
    while prv < cur:
        prv += 1
        stk.push(prv)
    let x = stk.pop()!
    if i < n: b[i] = x

print("YES")
print(b)

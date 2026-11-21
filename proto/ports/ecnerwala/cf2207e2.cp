// Codeforces 2207 E2: N-MEX (Counting Version).
type Z = Mod<1_000_000_007>

@!multitest
let n: Int = read()
var a: [0..=n]Int = n
for i in 1..=n: a[i] = read()

if not all(1..=n, i => a[i] in (n - i)..=a[i - 1]):
    print(0)
    return

var ans: Z = 1
var prv = a[n]
var pool = a[n]
for i in (0..n).rev:
    let cur = a[i]
    if cur > prv:
        pool += cur - prv - 1
        prv = cur
        ans *= pool + (n - cur + 1)
    else:
        ans *= pool
        pool -= 1
print(ans)

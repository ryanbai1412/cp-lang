const P = 1_000_000_007
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted(by: desc)
var ans = 1
for k in 0..n:
    if k > 0 and b[k] == b[k - 1]: break
    if k == n - 1 or b[k] > b[k + 1]:
        ans = (ans + pow_mod(b[k] - 1, k + 1, P)) % P
print(ans)

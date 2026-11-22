@!multitest
let n = read<1..=1e9>()
var ans = 0
for d in 1..=9:
    var x = d
    while x <= n:
        ans += 1
        x = x * 10 + d
print(ans)

@!multitest
let n = read<1..=1e9>()
var count = 0
for d in 1..=9:
    var x = d
    while x <= n:
        count += 1
        x = x * 10 + d
print(count)

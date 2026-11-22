// CF 1520B Ordinary Numbers: t tests, count x in 1..=n whose digits are all equal.
@!multitest
let n: 1..=1e9 = read()
var count = 0
for d in 1..=9:
    var x = d
    while x <= n:
        count += 1
        x = x * 10 + d
print(count)

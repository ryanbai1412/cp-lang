@!multitest
let n: Int = read()
let s: Str = read()
var lo = 1
var hi = n
var ans: List<Int> = []
for k in 0..n - 1:
    if s[n - 2 - k] == '>':
        ans.push(hi)
        hi -= 1
    else:
        ans.push(lo)
        lo += 1
ans.push(lo)
ans.reverse()
print(ans)

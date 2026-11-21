@!multitest
let n: Int = read()
let s: Str = read()
var lo = 1
var hi = n
var ans: [0..n]Int = 0
for i in (0..n - 1).rev:
    if s[i] == '>':
        ans[i + 1] = hi
        hi -= 1
    else:
        ans[i + 1] = lo
        lo += 1
ans[0] = lo
print(ans)

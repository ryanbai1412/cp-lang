@!multitest
let (n, x0): (Int, Int) = read()
let x = x0 - 1
let s: Str = read()
var l = x - 1
while l >= 0 and s[l] == '.': l -= 1
var r = x + 1
while r < n and s[r] == '.': r += 1
var ans = min(l + 2, n - r + 1)
if l < x - 1: ans max= min(x + 1, n - r + 1)
if r > x + 1: ans max= min(l + 2, n - x)
print(ans)

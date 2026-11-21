@!multitest
let (n, x0): (Int, Int) = read()
let x = x0 - 1
let s: Str = read()
let l = s[0..x].rfind('#') ?? -1
let r = first(i for i in x + 1..n if s[i] == '#') ?? n
var ans = min(l + 2, n - r + 1)
if l < x - 1: ans max= min(x + 1, n - r + 1)
if r > x + 1: ans max= min(l + 2, n - x)
print(ans)

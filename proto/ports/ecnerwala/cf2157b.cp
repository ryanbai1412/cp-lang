@!multitest
let (n, x0, y0): (Int, Int, Int) = read()
let s: Str = read()
var c4 = 0
var c8 = 0
for ch in s:
    if ch == '4': c4 += 1
    else: c8 += 1
let x = max(0, abs(x0) - c8)
let y = max(0, abs(y0) - c8)
print(if x + y <= c4: "YES" else: "NO")

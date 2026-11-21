@!multitest
let (n, x0, y0): (Int, Int, Int) = read()
let s: Str = read()
let c8 = s.count('8')
let c4 = n - c8
print(if max(0, abs(x0) - c8) + max(0, abs(y0) - c8) <= c4: "YES" else: "NO")

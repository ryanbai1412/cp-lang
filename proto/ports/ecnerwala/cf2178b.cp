@!multitest
let s: Str = read()
let inf = 1_000_000_000
var d0 = 0
var d1 = inf
for c in s:
    let m = min(d0, d1)
    if c == 's':
        d0 = inf
        d1 = m
    else:
        d0 = d1
        d1 = 1 + m
print(d1)

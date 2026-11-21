@!multitest
let s: Str = read()
var d0: ?Int = 0
var d1: ?Int = none
for c in s:
    let m = min(d0, d1)!
    if c == 's':
        d0 = none
        d1 = m
    else:
        d0 = d1
        d1 = 1 + m
print(d1!)

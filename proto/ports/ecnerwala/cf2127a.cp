@!multitest
let n: Int = read()
var v: ?Int = none
var bad = false
for _ in 0..n:
    let a: Int = read()
    if a >= 0:
        if let cur = v:
            if a != cur: bad = true
        else:
            v = a
if v == 0: bad = true
print(if bad: "NO" else: "YES")

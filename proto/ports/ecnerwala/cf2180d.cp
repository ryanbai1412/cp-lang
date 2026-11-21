@!multitest
let n: Int = read()
let x: [0..n]Int = read()
if n == 1:
    print(0)
    return

var v = 1
var (r_mi, r_ma) = (0, x[1] - x[0])
for i in 2..n:
    let d = x[i] - x[i - 1]
    r_ma min= d
    if r_ma <= r_mi:
        v -= 1
        (r_mi, r_ma) = (0, d)
    v += 1
    (r_mi, r_ma) = (d - r_ma, d - r_mi)
print(v)

@!multitest
let n: Int = read()
let x: [0..n]Int = read()
if n == 1:
    print(0)
else:
    var v = 1
    var r_mi = 0
    var r_ma = x[1] - x[0]
    for i in 2..n:
        let d = x[i] - x[i - 1]
        r_ma min= d
        if r_ma <= r_mi:
            v -= 1
            r_mi = 0
            r_ma = d
        v += 1
        let n_mi = d - r_ma
        let n_ma = d - r_mi
        r_mi = n_mi
        r_ma = n_ma
    print(v)

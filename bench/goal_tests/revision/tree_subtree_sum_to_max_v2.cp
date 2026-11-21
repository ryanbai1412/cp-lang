let n: Int = read()
let val: [0..n]Int = read()
var par: [0..n]Int = -1
for i in 1..n:
    par[i] = read()
var sub: [0..n]Int = 0
for i in 0..n:
    sub[i] = val[i]
for i in (1..n).rev:
    sub[par[i]] = max(sub[par[i]], sub[i])
print(sub[0])

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
let c: Str = read()

var x = 0
for v in a: x ^= v

var basis: [0..60]Int = 0
var val: [0..60]Bool = false
for i in (0..n).rev:
    var v = a[i] ^ b[i]
    for bit in (0..60).rev: v min= v ^ basis[bit]
    if v != 0:
        for bit in 0..60: basis[bit] min= basis[bit] ^ v
        let bit = bit_width(v) - 1
        basis[bit] = v
        val[bit] = c[i] == '1'

for bit in 0..60 if basis[bit] != 0:
    if val[bit]: x max= x ^ basis[bit]
    else: x min= x ^ basis[bit]
print(x)

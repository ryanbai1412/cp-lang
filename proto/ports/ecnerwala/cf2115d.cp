@!multitest
let n: Int = read()
var a: List<Int> = [read::<Int>() for _ in 0..n]
var x = 0
for v in a: x ^= v
for i in 0..n:
    let b: Int = read()
    a[i] ^= b
let c: Str = read()
var basis: [0..60]Int = 0
var val: [0..60]Int = -1
for i in (0..n).rev:
    var v = a[i]
    for b in (0..60).rev:
        v min= v ^ basis[b]
    if v != 0:
        for b in 0..60:
            basis[b] min= basis[b] ^ v
        let bit = bit_width(v) - 1
        basis[bit] = v
        val[bit] = Int(c[i] == '1')
for b in 0..60:
    if basis[b] != 0:
        if val[b] == 0: x min= x ^ basis[b]
        else: x max= x ^ basis[b]
print(x)

let (n, q, seed): (Int, Int, Int) = read()
domain D = 0..n
let mask = n - 1
var d = Diff<D, Int>()
var s = seed
for _ in 0..q:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let x = s & mask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let y = s & mask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let v = (s & 255) - 128
    d.add(D(min(x, y))..=D(max(x, y)), v)
let out = d.finish()
var acc = 0
for i in D:
    acc += out[i]
print(acc)

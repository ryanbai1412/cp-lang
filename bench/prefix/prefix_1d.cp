let (n, q, seed): (Int, Int, Int) = read()
domain D = 0..n
let mask = n - 1
var s = seed
var a: [D]Int = 0
for i in D:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    a[i] = (s & 2047) - 1024
let p = prefix(a)
var acc = 0
for _ in 0..q:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let x = s & mask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let y = s & mask
    let l = min(x, y)
    let r = max(x, y)
    acc += p.fold(D(l)..=D(r))
print(acc)

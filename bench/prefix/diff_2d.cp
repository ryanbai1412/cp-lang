let (h, w, q, seed): (Int, Int, Int, Int) = read()
domain R = 0..h
domain C = 0..w
let rmask = h - 1
let cmask = w - 1
var d = Diff<R, C, Int>()
var s = seed
for _ in 0..q:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let r1 = s & rmask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let r2 = s & rmask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let c1 = s & cmask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let c2 = s & cmask
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let v = (s & 31) - 16
    d.add(R(min(r1, r2))..=R(max(r1, r2)), C(min(c1, c2))..=C(max(c1, c2)), v)
let out = d.finish()
var acc = 0
for r in R:
    for c in C:
        acc += out[r, c]
print(acc)

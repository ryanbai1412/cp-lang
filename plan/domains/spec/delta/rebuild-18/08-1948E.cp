// CF1948E Clique Partition. Blocks of k consecutive vertices; inside a block
// of size m the values are h..1 then m..h+1 (h = ceil(m / 2)), so every pair
// has distance plus value gap at most m <= k.
@!multitest
let n: 2..=40 = read()
let k: 1..=80 = read()
var a: [n]Int = 0
var c: [n]Int = 0
var q = 0
var st = 0
while st < n:
    let m = min(k, n - st)
    let h = (m + 1) / 2
    for t in 0..m:
        a[st + t] = st + (if t < h: h - t else: m + h - t)
        c[st + t] = q + 1
    q += 1
    st += m
print(a)
print(q)
print(c)

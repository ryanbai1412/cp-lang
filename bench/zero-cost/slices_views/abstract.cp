fn kernel(n: Int, q: Int, a: List<Int>, qs: List<(Int, Int)>) -> Int:
    domain V = 0..n
    var arr: [V]Int = 0
    for i in V: arr[i] = a[i.to_idx]
    var acc = 0
    for (l, r) in qs:
        var s = 0
        for x in arr[V.nth(l)..V.nth(r)]:
            s = (s + x * 5) ^ (s >> 4)
        acc ^= s
    acc

let n: Int = read()
let q: Int = read()
let a: [0..n]Int = read()
var qs: List<(Int, Int)> = []
for _ in 0..q:
    let (l, r): (Int, Int) = read()
    qs.push((l, r))
print(kernel(n, q, a, qs))

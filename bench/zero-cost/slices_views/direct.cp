fn kernel(n: Int, q: Int, a: List<Int>, qs: List<(Int, Int)>) -> Int:
    var arr = [0 for _ in 0..n]
    for i in 0..n: arr[i] = a[i]
    var acc = 0
    for (l, r) in qs:
        var s = 0
        for i in l..r:
            s = (s + arr[i] * 5) ^ (s >> 4)
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

fn dot<D: Index>(a: [D]Int, b: [D]Int) -> Int:
    var s = 0
    for i in D: s += a[i] * b[i]
    s
let (n, m): (Int, Int) = read()
let a: [0..n]Int = 1
let b: [0..m]Int = 2
print(dot(a, b))

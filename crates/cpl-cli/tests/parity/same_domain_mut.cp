fn add<D: Index>(mut a: [D]Int, b: [D]Int):
    for i in D: a[i] += b[i]
let (n, m): (Int, Int) = read()
var a: [0..n]Int = 1
let b: [0..m]Int = 2
add(mut a, b)
print(a)

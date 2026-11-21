fn dot<D: Index>(a: [D]Int, b: [D]Int) -> Int:
    var s = 0
    for i in D: s += a[i] * b[i]
    s
fn add<D: Index>(mut a: [D]Int, b: [D]Int):
    for i in D: a[i] += b[i]
fn both<D: Index>(mut a: [D]Int, mut b: [D]Int) -> ?D:
    var at: ?D = none
    for i in D:
        a[i] += b[i]
        b[i] = 0
        at = i
    at
fn sum_arr<D: Index>(a: [D]Int) -> Int:
    var s = 0
    for i in D: s += a[i]
    s
let (n, m): (Int, Int) = read()
let a: [0..n]Int = 1
let b: [0..m]Int = 2
let l = [3, 4, 5, 6]
let six: [0..6]Int = [1, 2, 3, 4, 5, 6]
print(sum_arr(six[1..4]), sum_arr(l[1..3]), dot(l[0..2], l[2..4]))
var c: [1..=n]Int = 5
var d: [1..=n]Int = 7
print(both(mut c, mut d), c, d)
add(mut c, d)
print(dot(a, a), dot(l, [1, 1, 1, 1]))
var e: [0..m]Int = 1
add(mut e, b)
print(e)
print(dot(a, b))

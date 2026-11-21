// Domains whose bounds are constants get constexpr bounds in C++; a local
// that only sizes one stays declared for its other readers (a closure here).
let m = 7
domain V = 1..=m * 1000
let n: Int = read()
domain W = 0..n

fn f(k: Int) -> Int:
    let c = 5
    domain D = 0..c
    var a: [D]Int = 0
    let g = (x: Int) => x + c
    for d in D:
        a[d] = g(k)
    return a[D(0)] + D.K

fn depth(k: Int) -> Int:
    domain E = 1..=3
    var t: [E]Int = 0
    if k > 0:
        t[E(1)] = depth(k - 1)
    return t[E(1)] + E.K

var a: [V]Int = 0
var b: [W]Int = 0
a[V(m * 1000)] = 2
print(a[V(m * 1000)] + b[W(0)], 12345 % V.K, W.K)
print(f(2), depth(4))
for i in 0..2:
    let q = 4 + i
    domain F = 0..q
    print(F.K)

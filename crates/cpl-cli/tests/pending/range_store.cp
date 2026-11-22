struct S(a: 0..=1_000_000, b: Int)

let n: 1..=10 = read()
domain V = 0..n
let k: Int = read()

var x: 0..=100 = 0
x = k
x += 5
x max= 12
x min= 50

var s = S(k, -1)
s.a = k * 2 + 1

var d: [V]0..=1_000_000 = k
for v in V:
    d[v] += v.ord * 3

let comp: [V]0..=100 = [k + v.ord for v in V]

var xs: List<0..=100> = []
xs.push(k)
xs.push(k + 7)
xs.resize(4, k + 2)

print(x, s.a, s.b, d[V(0)], d[V(n - 1)], comp[V(n - 1)], xs[0], xs[1], xs[3])

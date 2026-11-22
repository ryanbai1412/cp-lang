domain V = 1..=5
var a: [V]Int = 0
var o: [0..6]?Int = none
var xs: List<?Int> = [some(1), some(2), none, some(4)]
a[2..4] = 7
print(a)
a[..] = 3
print(a)
o[1..4] = some(5)
print(o)
o[..2] = none
o[3..=4] = 8
print(o)
xs[1..3] = none
xs[..1] = some(9)
print(xs)
var g: [V, 0..3]?Int = none
g[V(2), ..] = some(1)
g[.., 1] = 4
print(g)
var m = Map<Str, Int>()
var h = HashMap<Int, Int>()
m["a"] = 1
m["b"] = 2
h[3] = 4
m.remove("a")
h.remove(3)
let gone = m.remove("zz")
print(m.len, h.len, gone)
print("hello".count('l'), "".count('x'))
let s = "hello"
let ys = [1, 2, 1]
print(s.count('l'), ys.count(1), "".count('x'), count(x == 1 for x in ys))

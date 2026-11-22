fn fmt_opt(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
domain V = 1..=5
var a: [V]Int = 0
var o: [0..6]?Int = none
var xs: List<?Int> = [some(1), some(2), none, some(4)]
a[2..4] = 7
print(a)
a[..] = 3
print(a)
o[1..4] = some(5)
print([fmt_opt(x) for x in o])
o[..2] = none
o[3..=4] = 8
print([fmt_opt(x) for x in o])
xs[1..3] = none
xs[..1] = some(9)
print([fmt_opt(x) for x in xs])
var g: [V, 0..3]?Int = none
g[V(2), ..] = some(1)
g[.., 1] = 4
for v in V: print([fmt_opt(g[v, j]) for j in 0..3])
var m = Map<Str, Int>()
var h = HashMap<Int, Int>()
m["a"] = 1
m["b"] = 2
h[3] = 4
m.remove("a")
h.remove(3)
let gone = m.remove("zz")
print(m.len, h.len, fmt_opt(gone))
print("hello".count('l'), "".count('x'))
let s = "hello"
let ys = [1, 2, 1]
print(s.count('l'), ys.count(1), "".count('x'), count(x for x in ys if x == 1))

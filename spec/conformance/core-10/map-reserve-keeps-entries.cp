// expect: ok, output "2 9\n1 5"
// rule: 10.3.8
// `reserve(n)` makes room for `n` keys in a hash map and keeps what is there; a sorted map takes it too.
var h: HashMap<Int, Int> = HashMap()
h[5] = 1
h.reserve(1000)
for i in 0..1000: h[i] = i * 2
h[3] = 9
print(h.len - 998, h[3])
var m: Map<Int, Int> = Map()
m[2] = 5
m.reserve(10)
print(m.len, m[2])

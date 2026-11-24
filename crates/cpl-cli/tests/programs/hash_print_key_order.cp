// A printed HashMap or HashSet shows its entries in key order on every build (decision 459).
struct P:
    a: Int
    b: Str
var h: HashMap<Int, Int> = []
for i in 0..20: h[(i * 7919) % 101] = i
print(h)
var s: HashSet<(Int, Int)> = []
for i in 0..10: s.add((i % 3, 10 - i))
print(s)
var hp: HashMap<P, Int> = []
hp[P(2, "x")] = 1
hp[P(1, "y")] = 2
hp[P(1, "b")] = 3
print(hp)
var hs: HashSet<Str> = []
for w in ["pear", "fig", "apple", "kiwi"]: hs.add(w)
print(hs)

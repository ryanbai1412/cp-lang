// expect: ok, output "2 5\n1"
// rule: 10.3.8
// A hash container prints through an order the program chooses.
var h: HashMap<Int, Int> = HashMap()
h[5] = 1
h[2] = 7
print(h.keys.sorted())
var s: HashSet<Int> = HashSet()
s.add(3)
print(s.len)

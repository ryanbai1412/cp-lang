// expect: ok, output "6\n0\n3\n7"
// rule: 07.12.9
// `if m.contains(k): m[k] op= v` stores into the entry it found, whatever path names the map,
// and leaves an absent key absent.
struct B(h: Int, len: Int)
var maps: [1..=3]HashMap<Int, Int> = []
maps[2][5] = 0
let b = B(5, 2)
let c = B(9, 2)
for i in 0..4:
    if maps[b.len].contains(b.h): maps[b.len][b.h] += i
    if maps[c.len].contains(c.h): maps[c.len][c.h] += i
print(maps[2][5])
print(maps[2].len - 1)
var t = Map<Str, Int>()
t["x"] = 3
let k = "x"
if t.contains(k): t[k] max= 2
print(t["x"])
if t.contains(k): t[k] = 7
print(t["x"])

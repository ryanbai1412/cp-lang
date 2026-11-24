// `place op= value` runs on the VM at any place: a field or a whole local
// held in integer registers, a Bool cell, a string's char, a U64 local's
// `min=`, and an entry of a map ordered by a user `Ord`.
struct Run:
    var head: Int
    var tail: Int
    var odd: Bool
fn walk(n: Int) -> Int:
    var r = Run(head: 0, tail: n, odd: false)
    var best = (0, 0)
    var low = (n, n, n)
    for i in 0..n:
        r.tail -= i
        r.head += 1
        r.odd = not r.odd
        best max= (i % 3, i)
        low min= (n - i, i % 2, i)
    print(r.head, r.tail, yes(r.odd), best, low)
    r.tail
var g: [3][3]Bool = false
var row: [4]Bool = false
for i in 0..5:
    g[i % 3, (i + 1) % 3] ^= true
    row[i % 4] ^= true
print([yes(g[i, j]) for i in 0..3 for j in 0..3], sep: " ")
print([yes(b) for b in row], sep: " ")
var s = "abcd"
for i in 0..4: s[i] = Char('z'.ord - i)
print(s)
var u: U64 = 9
for x in [5, 7, 3]: u min= U64(x)
print(u)
var m = Map<Desc<Int>, Int>()
for k in [2, 1, 2, 3]:
    m[desc(k)] += k
    m[desc(k + 10)] max= k
m[desc(5)] -= 4
print([(k.x, v) for (k, v) in m.items], sep: " ")
print(walk(6))

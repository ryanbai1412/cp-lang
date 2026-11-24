var m = Map<Int, List<Int>>()
m[0] = [1, 2, 3]
if let l = m.get(0):
    m[0] = []
    print(l.len)
while let w = m.get(0):
    print(w.len)
    m.remove(0)
print(m.len)
var h = HashMap<Int, List<Int>>()
h[1] = [5, 6]
var s = 0
for i in 0..4:
    if let l = h.get(1):
        s += l.len + l[0]
    if let l = h.get(2):
        s += 100 + l.len
print(s)
fn total(m: Map<Int, List<Int>>, fallback: List<Int>) -> Int:
    var t = 0
    for i in 0..3:
        let l = m.get(i) ?? fallback
        let k = m.get(i + 1) ?? [i, i]
        t += l.len * 10 + l[0] + k.len
    t
var c = Map<Int, List<Int>>()
c[0] = [4, 5]
c[2] = [6]
print(total(c, [9]))
let before = c.get(0) ?? []
c[0] = []
var grown = c.get(7) ?? [1]
grown.push(2)
print(before.len, c[0].len, grown.len)
var w = Map<Int, List<Int>>()
w[0] = [1, 2, 3]
var t = 0
for k in 0..2:
    for v in w.get(k) ?? [7]: t += v
for v in w.get(0) ?? []: w[0] = [v * 10]
print(t, w[0])
fn named(m: Map<Int, List<Int>>, k: Int) -> Int:
    let o = m.get(k)
    var s = 0
    if let l = o: s += l.len
    for v in o ?? [9]: s += v
    if o.is_some: s += 1
    return s
print(named(w, 0), named(w, 5))
var z = Map<Int, List<Int>>()
z[0] = [4]
let held = z.get(0)
z[0] = [5]
print(held ?? [], held.is_some)
fn changed() -> Int:
    var z = Map<Int, List<Int>>()
    z[0] = [4]
    let held = z.get(0)
    z[0] = [5]
    return (held ?? []).sum
print(changed())
var pm = Map<Int, List<Int>>()
pm[0] = [1, 5, 9]
var cut = 0
for hi in [2, 6, 10]:
    let l = pm.get(0) ?? []
    cut += l.partition_point(x => x >= hi)
var keys: List<Int> = [3, 1, 2]
keys.sort()
print(cut, keys.partition_point(k => k >= 3))
var seen = 0
for k in 0..2:
    let l = pm.get(0) ?? [7]
    let grow = (x: Int) => pm[x + 1] = [x]
    grow(k)
    seen += l.len + pm.len
print(seen)
var sh = [Map<Int, List<Int>>(), Map<Int, List<Int>>()]
sh[1][3] = [4, 5]
var walked = 0
for b in 0..2:
    for i in sh[b].get(3) ?? [9]:
        walked += i
print(walked)
fn summed(xs: List<Int>) -> Int:
    var s = 0
    for x in xs: s += x
    s
var by_key = Map<Int, List<Int>>()
by_key[1] = [2, 3]
print(summed(by_key.get(1) ?? []), summed(by_key.get(5) ?? [7]))
var sorted_in = Map<Int, List<Int>>()
for v in 0..4: sorted_in[v % 2] = (sorted_in.get(v % 2) ?? []) + [v]
print(summed(sorted_in.get(0) ?? []), summed(sorted_in.get(1) ?? []), sorted_in.len)
let marks = [3, 1, 3, 2, 1]
var grouped = Map<Int, List<Int>>()
for v in 0..5: grouped[marks[v]] = (grouped.get(marks[v]) ?? []) + [v]
for v in 0..5: grouped[marks[v]] = (grouped.get(marks[v]) ?? []) + grouped[1]
print(grouped.get(3) ?? [])
print(grouped.get(1) ?? [])

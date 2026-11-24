// HashMap and HashSet through many rehashes: inserts, overwrites, removals
// (each shifting a probe run back), lookups of absent keys, reserve and
// equality of tables built in different orders.
var m: HashMap<Int, Int> = []
var s: HashSet<Int> = HashSet()
var x = 12345
var hits = 0
var total = 0
for step in 0..200_000:
    x = (x * 1103515245 + 12345) % 2_147_483_648
    let k = x % 5000
    if step % 3 == 0:
        m.remove(k)
        s.remove(k)
    else:
        m[k] = (m.get(k) ?? 0) + step
        s.add(k)
    if let v = m.get(x % 7000):
        hits += 1
        total = (total + v) % 1_000_000_007
print(m.len, s.len, hits, total)
var a: HashMap<Int, Int> = []
var b: HashMap<Int, Int> = []
b.reserve(3000)
for i in 0..3000: a[i * 7919 % 3001] = i
for i in 0..3000: b[(2999 - i) * 7919 % 3001] = 2999 - i
print(if a == b: 1 else: 0, a.len)
b[0] = -1
print(if a == b: 1 else: 0)
var w: HashMap<Str, Int> = []
for i in 0..20_000: w["k{i % 777}"] = (w.get("k{i % 777}") ?? 0) + i
for i in 0..777:
    if i % 2 == 0: w.remove("k{i}")
print(w.len, w.get("k1") ?? 0, w.get("k2") ?? -1)

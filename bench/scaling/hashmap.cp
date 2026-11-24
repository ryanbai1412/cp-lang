use std.hashmap.{FlatMap, Counter}
let n: Int = read()
let reps = 1 << 18
var s = 0
let xs = [i * 7919 % n for i in 0..n]

for _ in 0..400:
    let m = FlatMap(0, cap: n) // timed: hashmap.FlatMap.new
    s += m.len
var m = FlatMap(0)
for i in 0..reps:
    m.set(i * 7919 % n, i) // timed: hashmap.FlatMap.set
    s += m.get(i * 13 % n) ?? 0 // timed: hashmap.FlatMap.get
    if m.contains(i * 17 % n): s += 1 // timed: hashmap.FlatMap.contains
    s += m.get_or_insert(i * 19 % n, 1) // timed: hashmap.FlatMap.get_or_insert
    s += m.remove(i * 31 % n) ?? 0 // timed: hashmap.FlatMap.remove
    s += m.len // timed: hashmap.FlatMap.len
    if m.is_empty: s += 1 // timed: hashmap.FlatMap.is_empty
for i in 0..n: m.set(i, i)
for _ in 0..40:
    for (k, v) in m.items: // timed: hashmap.FlatMap.items
        s += k + v // timed: hashmap.FlatMap.items
    for k in m.keys: // timed: hashmap.FlatMap.keys
        s += k // timed: hashmap.FlatMap.keys
    for v in m.values: // timed: hashmap.FlatMap.values
        s += v // timed: hashmap.FlatMap.values
for _ in 0..200:
    var a = FlatMap(0)
    var b = FlatMap(0)
    for i in 0..n:
        a.set(i, i)
        b.set(i + n / 2, i)
    a.absorb(mut b, join: (x, y) => x + y) // timed: hashmap.FlatMap.absorb
    s += a.len
    a.reserve(4 * n) // timed: hashmap.FlatMap.reserve
    var t = a.take()
    t.clear() // timed: hashmap.FlatMap.clear
    s += t.len

for _ in 0..200:
    let c = Counter(xs) // timed: hashmap.Counter.new
    s += c.len
    let d = Counter.of(xs) // timed: hashmap.Counter.of
    s += d.total() // timed: hashmap.Counter.total
for _ in 0..20:
    s += Counter.of(xs).most_common(n).len // timed: hashmap.Counter.most_common
var c: Counter<Int, Int> = Counter()
for i in 0..reps:
    c.add(i * 7919 % n, 2) // timed: hashmap.Counter.add
    c[i * 11 % n] = 3 // timed: hashmap.Counter.set
    s += c[i * 13 % n] // timed: hashmap.Counter.get
    if c.contains(i * 17 % n): s += 1 // timed: hashmap.Counter.contains
    s += c.remove(i * 31 % n) ?? 0 // timed: hashmap.Counter.remove
    c.push(i * 37 % n) // timed: hashmap.push
    c.pop(i * 41 % n) // timed: hashmap.pop
    s += c.len // timed: hashmap.Counter.len
    if c.is_empty: s += 1 // timed: hashmap.Counter.is_empty
for i in 0..n: c.add(i, 1)
for _ in 0..40:
    for (k, v) in c.items: // timed: hashmap.Counter.items
        s += k + v // timed: hashmap.Counter.items
    for k in c.keys: // timed: hashmap.Counter.keys
        s += k // timed: hashmap.Counter.keys
    for v in c.values: // timed: hashmap.Counter.values
        s += v // timed: hashmap.Counter.values
for _ in 0..200:
    var a = Counter.of(xs)
    var b = Counter([x + n / 2 for x in xs])
    a.absorb(mut b) // timed: hashmap.Counter.absorb
    a.reserve(4 * n) // timed: hashmap.Counter.reserve
    var t = a.take()
    t.clear() // timed: hashmap.Counter.clear
    s += t.len
var tf = FlatMap(0)
tf.set(1, 1)
var tc = Counter.of([1])
for _ in 0..1 << 20:
    var x = tf.take() // timed: hashmap.FlatMap.take
    tf = x.take()
    var y = tc.take() // timed: hashmap.Counter.take
    tc = y.take()
print(s)

// A SortedMap of n keys: queries, then edits that undo themselves, then whole-map moves.
let n: Int = read()
var m: SortedMap<Int, Int> = SortedMap()
for i in 0..n:
    m.set(i * 7919 % n, i) // timed: bag.SortedMap.set
var s = 0
let small = (k: Int, v: Int) => k < n / 2 // within: bag.SortedMap.first_true
let big = (k: Int, v: Int) => k < n / 2 // within: bag.SortedMap.last_true
for i in 0..4 * n:
    let k = i * 13 % n
    s += m.get(k) ?? 0 // timed: bag.SortedMap.get
    if m.contains(k): s += 1 // timed: bag.SortedMap.contains
    s += m.rank(k) // timed: bag.SortedMap.rank
    s += (m.nth(k) ?? (0, 0)).0 // timed: bag.SortedMap.nth
    s += (m.first_ge(k) ?? (0, 0)).0 // timed: bag.SortedMap.first_ge
    s += (m.first_gt(k) ?? (0, 0)).0 // timed: bag.SortedMap.first_gt
    s += (m.last_le(k) ?? (0, 0)).0 // timed: bag.SortedMap.last_le
    s += (m.last_lt(k) ?? (0, 0)).0 // timed: bag.SortedMap.last_lt
    s += (m.first_true(small) ?? (0, 0)).0 // timed: bag.SortedMap.first_true
    s += (m.last_true(big) ?? (0, 0)).0 // timed: bag.SortedMap.last_true
    let v = m.remove(k)! // timed: bag.SortedMap.remove
    m.set(k, v)
    let f = m.pop_first()! // timed: bag.SortedMap.pop_first
    m.set(f.0, f.1)
    let l = m.pop_last()! // timed: bag.SortedMap.pop_last
    m.set(l.0, l.1)
for i in 0..64 * n:
    s += m.len // timed: bag.SortedMap.len
    if m.is_empty: s += 1 // timed: bag.SortedMap.is_empty
    s += (m.first ?? (0, 0)).0 // timed: bag.SortedMap.first
    s += (m.last ?? (0, 0)).0 // timed: bag.SortedMap.last
    var e: SortedMap<Int, Int> = SortedMap() // timed: bag.SortedMap.new
    e.set(i, i)
    let u = e.take() // timed: bag.SortedMap.take
    s += u.len
    e.set(i, i)
    e.clear() // timed: bag.SortedMap.clear
for r in 0..40960:
    var f: SortedMap<Int, Int> = SortedMap()
    f.reserve(n) // timed: bag.SortedMap.reserve
    s += f.len
for r in 0..16:
    for (k, v) in m.items(): // timed: bag.SortedMap.items
        s += k ^ v // timed: bag.SortedMap.items
    for (k, v) in m.each_rev(): // timed: bag.SortedMap.each_rev
        s += k // timed: bag.SortedMap.each_rev
    for k in m.keys(): // timed: bag.SortedMap.keys
        s ^= k // timed: bag.SortedMap.keys
    for v in m.values(): // timed: bag.SortedMap.values
        s += v // timed: bag.SortedMap.values
    var w: SortedMap<Int, Int> = SortedMap()
    for i in 0..n: w.set(i, i)
    let d = w.drain(after: n / 4, while: (k, v) => k < n / 2) // timed: bag.SortedMap.drain
    s += d.len
    var hi = w.split_at(n / 2) // timed: bag.SortedMap.split_at
    w.join(hi) // timed: bag.SortedMap.join
    var o: SortedMap<Int, Int> = SortedMap()
    for i in 0..n: o.set(i + n / 2, i)
    w.absorb(mut o, join: (x, y) => x + y) // timed: bag.SortedMap.absorb
    s += w.len
print(s)

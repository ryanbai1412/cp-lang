// A SortedSet of n keys: queries, then edits that undo themselves, then whole-set moves.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var t = SortedSet<Int>.of(xs) // timed: bag.SortedSet.of
var s = 0
let small = (x: Int) => x < n / 2 // within: bag.SortedSet.first_true
let big = (x: Int) => x < n / 2 // within: bag.SortedSet.last_true
for i in 0..4 * n:
    let k = i * 13 % n
    if t.contains(k): s += 1 // timed: bag.SortedSet.contains
    s += t.rank(k) // timed: bag.SortedSet.rank
    s += t.nth(k) ?? 0 // timed: bag.SortedSet.nth
    s += t.first_true(small) ?? 0 // timed: bag.SortedSet.first_true
    s += t.last_true(big) ?? 0 // timed: bag.SortedSet.last_true
    s += t.range(k..k + 8).len // timed: bag.SortedSet.range
    if t.remove(k): s += 1 // timed: bag.SortedSet.remove
    if t.add(k): s += 1 // timed: bag.SortedSet.add
    t.set(k, false) // timed: bag.SortedSet.set
    t.set(k, true) // timed: bag.SortedSet.set
    let f = t.pop_first()! // timed: bag.SortedSet.pop_first
    t.add(f)
    let l = t.pop_last()! // timed: bag.SortedSet.pop_last
    t.add(l)
for i in 0..64 * n:
    s += t.len // timed: bag.SortedSet.len
    if t.is_empty: s += 1 // timed: bag.SortedSet.is_empty
    s += t.first ?? 0 // timed: bag.SortedSet.first
    s += t.last ?? 0 // timed: bag.SortedSet.last
    var e = SortedSet<Int>() // timed: bag.SortedSet.new
    e.add(i)
    let u = e.take() // timed: bag.SortedSet.take
    s += u.len
    e.add(i)
    e.clear() // timed: bag.SortedSet.clear
for r in 0..128:
    var f = SortedSet<Int>.of(xs)
    f.reserve(2 * n) // timed: bag.SortedSet.reserve
    s += f.len
let ys = [i * 31 % n + n / 2 for i in 0..n]
for r in 0..16:
    for x in t.each(): // timed: bag.SortedSet.each
        s += x // timed: bag.SortedSet.each
    for x in t.each_rev(): // timed: bag.SortedSet.each_rev
        s ^= x // timed: bag.SortedSet.each_rev
    s += t.to_list().len // timed: bag.SortedSet.to_list
    var w = SortedSet<Int>.of(xs)
    let d = w.drain(after: n / 4, while: x => x < n / 2) // timed: bag.SortedSet.drain
    s += d.len
    var hi = w.split_at(n / 2) // timed: bag.SortedSet.split_at
    w.join(hi) // timed: bag.SortedSet.join
    var o = SortedSet<Int>.of(ys)
    w.absorb(mut o) // timed: bag.SortedSet.absorb
    s += w.len
print(s)

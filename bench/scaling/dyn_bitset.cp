use std.bitset.{DynBitset}
let n: Int = read()
var acc = 0
var b = DynBitset(n)
for i in 0..n:
    if i % 3 == 0: b.add(i)
var hi = DynBitset(n)
hi.add(n - 1)
var lo = DynBitset(n)
lo.add(0)
let none = DynBitset(n)
var ws: List<Int> = []
var sum = 0
while sum < n:
    ws.push(ws.len + 1)
    sum += ws.len
var t = DynBitset(n)
for i in 0..6 * n:
    let d = i % n
    if b.contains(d): acc += 1 // timed: bitset.DynBitset.contains
    if t.add(d): acc += 1 // timed: bitset.DynBitset.add
    t.toggle(i * 7 % n) // timed: bitset.DynBitset.toggle
    if t.remove(d): acc += 1 // timed: bitset.DynBitset.remove
    t.set(i * 3 % n, i % 2 == 0) // timed: bitset.DynBitset.set
    acc += b.len // timed: bitset.DynBitset.len
    acc += b.cap // timed: bitset.DynBitset.cap
    if b.is_empty: acc += 1 // timed: bitset.DynBitset.is_empty
for i in 0..2 * n:
    var c = DynBitset.full(n)
    acc += DynBitset(n).len // timed: bitset.DynBitset.new
    acc += DynBitset.full(n).len // timed: bitset.DynBitset.full
    c.clear() // timed: bitset.DynBitset.clear
    acc += hi.first() ?? 0 // timed: bitset.DynBitset.first
    acc += lo.last() ?? 0 // timed: bitset.DynBitset.last
    acc += hi.next(0) ?? 0 // timed: bitset.DynBitset.next
    acc += lo.prev(n - 1) ?? 0 // timed: bitset.DynBitset.prev
    if b.meets(none): acc += 1 // timed: bitset.DynBitset.meets
    acc += c.len
for i in 0..n / 4:
    var c = DynBitset.full(n)
    c.resize(n - i % 2) // timed: bitset.DynBitset.resize
    c.resize(n)
    acc += b.count_below(n - 1) // timed: bitset.DynBitset.count_below
    acc += b.count_range(1..n - 1) // timed: bitset.DynBitset.count_range
    acc += hi.nth(0) ?? 0 // timed: bitset.DynBitset.nth
    c.union(b) // timed: bitset.DynBitset.union
    c.intersect(hi) // timed: bitset.DynBitset.intersect
    c.subtract(lo) // timed: bitset.DynBitset.subtract
    c.symmetric(b) // timed: bitset.DynBitset.symmetric
    c.shift_up(i % 70) // timed: bitset.DynBitset.shift_up
    c.shift_down(i % 70) // timed: bitset.DynBitset.shift_down
    c.shift_or(i % 70) // timed: bitset.DynBitset.shift_or
    acc += c.len
for _ in 0..384:
    for x in b.each(): acc += x // timed: bitset.DynBitset.each
    for x in b.each_rev(): acc ^= x // timed: bitset.DynBitset.each_rev
for _ in 0..64:
    acc += DynBitset.subset_sums(n, ws).len // timed: bitset.DynBitset.subset_sums
print(acc)

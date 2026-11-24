use std.bitset.{Bitset}
let n: Int = read()
domain D = 0..n
var acc = 0
var b = Bitset<D>()
for i in 0..n:
    if i % 3 == 0: b.add(D(i))
let hi = Bitset<D>.single(D(n - 1))
let lo = Bitset<D>.single(D(0))
let none = Bitset<D>()
let xs = [D(i * 7 % n) for i in 0..n / 2]
let flags = [D] d => d.ord % 5 == 0
var ws: List<Int> = []
var sum = 0
while sum < n:
    ws.push(ws.len + 1)
    sum += ws.len
var t = Bitset<D>()
for i in 0..6 * n:
    let d = D(i % n)
    if b.contains(d): acc += 1 // timed: bitset.Bitset.contains
    if t.add(d): acc += 1 // timed: bitset.Bitset.add
    t.toggle(D(i * 7 % n)) // timed: bitset.Bitset.toggle
    if t.remove(d): acc += 1 // timed: bitset.Bitset.remove
    acc += b.len // timed: bitset.Bitset.len
    acc += b.count // timed: bitset.Bitset.count
    if b.is_empty: acc += 1 // timed: bitset.Bitset.is_empty
for i in 0..2 * n:
    let d = D(i * 13 % n)
    var c = b.complement()
    acc += Bitset<D>().len // timed: bitset.Bitset.new
    acc += Bitset<D>.empty().len // timed: bitset.Bitset.empty
    acc += Bitset<D>.full().len // timed: bitset.Bitset.full
    acc += Bitset<D>.single(d).len // timed: bitset.Bitset.single
    acc += b.with(d).len // timed: bitset.Bitset.with
    acc += b.without(d).len // timed: bitset.Bitset.without
    acc += b.set(d, i % 2 == 0).len // timed: bitset.Bitset.set
    c.clear() // timed: bitset.Bitset.clear
    acc += (hi.first() ?? d).ord // timed: bitset.Bitset.first
    acc += (lo.prev(D(n - 1)) ?? d).ord // timed: bitset.Bitset.prev
    c.union(b) // timed: bitset.Bitset.union
    c.intersect(hi) // timed: bitset.Bitset.intersect
    c.subtract(lo) // timed: bitset.Bitset.subtract
    c.symmetric(b) // timed: bitset.Bitset.symmetric
    if b.meets(none): acc += 1 // timed: bitset.Bitset.meets
    c.shift_up(i % 70) // timed: bitset.Bitset.shift_up
    c.shift_or(i % 70) // timed: bitset.Bitset.shift_or
    c.union_shifted(b, i % 70) // timed: bitset.Bitset.union_shifted
    acc += (hi.next(D(0)) ?? d).ord // timed: bitset.Bitset.next
    acc += c.len
for i in 0..n / 2:
    let d = D(i * 13 % n)
    var c = b.complement()
    acc += b.complement().len // timed: bitset.Bitset.complement
    if hi.any(x => x.ord > 0): acc += 1 // timed: bitset.Bitset.any
    acc += (hi.find(x => x.ord > 0) ?? d).ord // timed: bitset.Bitset.find
    acc += (lo.rfind(x => x.ord < n) ?? d).ord // timed: bitset.Bitset.rfind
    acc += (hi.nth(0) ?? d).ord // timed: bitset.Bitset.nth
    acc += b.count_below(D(n - 1)) // timed: bitset.Bitset.count_below
    acc += (b | hi).len // timed: bitset.Bitset.bitor
    acc += (b & c).len // timed: bitset.Bitset.bitand
    acc += (b ^ c).len // timed: bitset.Bitset.bitxor
    acc += b.shifted(i % 100 - 50).len // timed: bitset.Bitset.shifted
    c.shift_down(i % 70) // timed: bitset.Bitset.shift_down
    acc += b.count_range(D(1)..D(n - 1)) // timed: bitset.Bitset.count_range
    c.set_range(D(1)..D(n - 1), on: i % 2 == 0) // timed: bitset.Bitset.set_range
    c.flip_range(D(1)..D(n - 1)) // timed: bitset.Bitset.flip_range
    acc += c.len
for _ in 0..384:
    acc += b.each().count() // timed: bitset.Bitset.each
    acc += b.each_rev().count() // timed: bitset.Bitset.each_rev
    acc += Bitset<D>.of(xs).len // timed: bitset.Bitset.of
    acc += Bitset<D>.from(flags).len // timed: bitset.Bitset.from
for _ in 0..64:
    acc += Bitset<D>.subset_sums(ws).len // timed: bitset.Bitset.subset_sums
print(acc)

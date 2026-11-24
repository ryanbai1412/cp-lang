// The point queries of a SortedBag of n keys, two copies of each.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n] + [i for i in 0..n]
var b = SortedBag<Int>.of(xs) // timed: bag.SortedBag.of
var s = 0
let small = (x: Int) => x < n / 2 // within: bag.SortedBag.first_true
let big = (x: Int) => x < n / 2 // within: bag.SortedBag.last_true
for i in 0..4 * n:
    let k = i * 13 % n
    s += b.rank(k) // timed: bag.SortedBag.rank
    s += b.nth(i % (2 * n)) ?? 0 // timed: bag.SortedBag.nth
    s += b.first_true(small) ?? 0 // timed: bag.SortedBag.first_true
    s += b.last_true(big) ?? 0 // timed: bag.SortedBag.last_true
    let c = b.cursor_at(k)! // timed: bag.SortedBag.cursor_at
    s += b.key_of(c)
    if let d = b.next_cursor(c): s += b.key_of(d) // timed: bag.SortedBag.next_cursor
    if let d = b.prev_cursor(c): s += b.key_of(d) // timed: bag.SortedBag.prev_cursor
    if let d = b.cursor_ge(k): s += b.key_of(d) // timed: bag.SortedBag.cursor_ge
    if let d = b.cursor_gt(k): s += b.key_of(d) // timed: bag.SortedBag.cursor_gt
    if let d = b.cursor_le(k): s += b.key_of(d) // timed: bag.SortedBag.cursor_le
    if let d = b.cursor_lt(k): s += b.key_of(d) // timed: bag.SortedBag.cursor_lt
    if let d = b.cursor_first(): s += b.key_of(d) // timed: bag.SortedBag.cursor_first
    if let d = b.cursor_last(): s += b.key_of(d) // timed: bag.SortedBag.cursor_last
    s += b.range(k..k + 8).len // timed: bag.SortedBag.range
let c0 = b.cursor_first()!
for i in 0..64 * n:
    s += b.len // timed: bag.SortedBag.len
    s += b.distinct // timed: bag.SortedBag.distinct
    if b.is_empty: s += 1 // timed: bag.SortedBag.is_empty
    s += b.first ?? 0 // timed: bag.SortedBag.first
    s += b.last ?? 0 // timed: bag.SortedBag.last
    s += b.key_of(c0) // timed: bag.SortedBag.key_of
    s += b.count_of(c0) // timed: bag.SortedBag.count_of
for r in 0..160:
    s += b.to_list().len // timed: bag.SortedBag.to_list
for r in 0..16:
    for x in b.each(): // timed: bag.SortedBag.each
        s += x // timed: bag.SortedBag.each
    for x in b.each_rev(): // timed: bag.SortedBag.each_rev
        s ^= x // timed: bag.SortedBag.each_rev
    for e in b.entries(): // timed: bag.SortedBag.entries
        s += e.count // timed: bag.SortedBag.entries
    for e in b.entries_rev(): // timed: bag.SortedBag.entries_rev
        s ^= e.key // timed: bag.SortedBag.entries_rev
for r in 0..256:
    s += b.sum() // timed: bag.SortedBag.sum
print(s)

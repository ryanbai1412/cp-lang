// AugSortedBag and AugSortedSet of n keys summed by `+`: queries, edits that undo themselves.
use std.bag.{AugSortedBag, AugSortedSet}
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
var t: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
for x in xs:
    b.add(x) // timed: bag.AugSortedBag.add
    if t.add(x): s += 1 // timed: bag.AugSortedSet.add
for i in 0..4 * n:
    let k = i * 13 % n
    s += b.count(k) // timed: bag.AugSortedBag.count
    if t.contains(k): s += 1 // timed: bag.AugSortedSet.contains
    s += b.rank(k) // timed: bag.AugSortedBag.rank
    s += t.rank(k) // timed: bag.AugSortedSet.rank
    s += b.nth(k) ?? 0 // timed: bag.AugSortedBag.nth
    s += t.nth(k) ?? 0 // timed: bag.AugSortedSet.nth
    s += b.fold_lt(k) // timed: bag.AugSortedBag.fold_lt
    s += t.fold_lt(k) // timed: bag.AugSortedSet.fold_lt
    s += b.fold_le(k) // timed: bag.AugSortedBag.fold_le
    s += t.fold_le(k) // timed: bag.AugSortedSet.fold_le
    s += b.fold_ge(k) // timed: bag.AugSortedBag.fold_ge
    s += t.fold_ge(k) // timed: bag.AugSortedSet.fold_ge
    s += b.fold_gt(k) // timed: bag.AugSortedBag.fold_gt
    s += t.fold_gt(k) // timed: bag.AugSortedSet.fold_gt
    s += b.fold(k / 2..k) // timed: bag.AugSortedBag.fold
    s += t.fold(k / 2..k) // timed: bag.AugSortedSet.fold
    s += b.fold_between(k / 2, k) // timed: bag.AugSortedBag.fold_between
    s += t.fold_between(k / 2, k) // timed: bag.AugSortedSet.fold_between
    if b.remove(k): s += 1 // timed: bag.AugSortedBag.remove
    b.add(k)
    if t.remove(k): s += 1 // timed: bag.AugSortedSet.remove
    t.add(k)
for i in 0..64 * n:
    s += b.len // timed: bag.AugSortedBag.len
    s += t.len // timed: bag.AugSortedSet.len
    if b.is_empty: s += 1 // timed: bag.AugSortedBag.is_empty
    if t.is_empty: s += 1 // timed: bag.AugSortedSet.is_empty
    s += b.all // timed: bag.AugSortedBag.all
    s += t.all // timed: bag.AugSortedSet.all
    s += b.first ?? 0 // timed: bag.AugSortedBag.first
    s += t.first ?? 0 // timed: bag.AugSortedSet.first
    s += b.last ?? 0 // timed: bag.AugSortedBag.last
    s += t.last ?? 0 // timed: bag.AugSortedSet.last
    var e: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k) // timed: bag.AugSortedBag.new
    var f: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k) // timed: bag.AugSortedSet.new
    s += e.len + f.len
for r in 0..16:
    for x in b.each(): // timed: bag.AugSortedBag.each
        s += x // timed: bag.AugSortedBag.each
    for x in t.each(): // timed: bag.AugSortedSet.each
        s ^= x // timed: bag.AugSortedSet.each
print(s)

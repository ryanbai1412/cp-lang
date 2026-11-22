// expect: ok, output "0 YES YES\n7 3 2 1 3 YES 1 3\n1 7 6 6 0 1 6\nYES YES\nYES\nYES NO YES YES 2 7 5 YES"
use std.bag.{AugSortedBag, AugSortedSet}
fn show<T>(o: ?T) -> Str = match o:
    Some(x) => "{x}"
    None => "none"
var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
print(b.all, yes(b.is_empty), yes(b.first == None))
b.add(3)
b.add(1)
b.add(3)
print(b.all, b.len, b.count(3), b.rank(3), show(b.nth(2)), yes(b.nth(3) == None), show(b.first), show(b.last))
print(b.fold_lt(3), b.fold_le(3), b.fold_ge(2), b.fold_gt(1), b.fold_gt(3), b.fold_between(1, 3), b.fold_between(2, 4))
print(yes(b.each().collect() == [1, 3, 3]), yes(b.remove(3) and b.all == 4 and not b.remove(2)))
var s: AugSortedBag<Int, Str> = AugSortedBag((x, y) => x + y, "", k => Str(k))
var ref: List<Int> = []
var seed = 11
var ok = true
for step in 0..600:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let x = seed % 13
    if seed / 13 % 3 == 0:
        let had = ref.contains(x)
        if s.remove(x) != had: ok = false
        if had:
            var i = 0
            while ref[i] != x: i += 1
            let _ = ref.remove(i)
    else:
        s.add(x)
        ref.push(x)
        ref.sort()
    let lo = seed % 7
    let hi = seed % 11 + 2
    var want = ""
    for y in ref:
        if lo <= y and y < hi: want = want + Str(y)
    if s.fold_between(lo, hi) != want: ok = false
    var all = ""
    for y in ref: all = all + Str(y)
    if s.all != all or s.len != ref.len: ok = false
    if s.each().collect() != ref: ok = false
    if ref.len > 0 and (s.first != Some(ref[0]) or s.last != Some(ref[ref.len - 1]) or s.nth(step % ref.len) != Some(ref[step % ref.len])): ok = false
print(yes(ok))
var t: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
print(yes(t.add(5)), yes(t.add(5)), yes(t.add(2)), yes(t.contains(5)), t.len, t.all, t.fold_ge(3), yes(t.each().collect() == [2, 5]))

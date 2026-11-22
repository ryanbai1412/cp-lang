use std.seq.{chunks, deduped, is_sorted, pairs, runs, windows}

// Sequence method parity across List<T>, [D]T, Str, ListView (xs[l..r]), and StrView (s[l..r]).
let xs: List<Int> = [3, 1, 2, 1, 3]
var a: [0..5]Int = 0
for i in 0..5: a[i] = xs[i]
let s: Str = "cabac"

// map (including `xs.map(c => Int(c == 'B'))`)
print("ABBA".map(c => Int(c == 'B')))
print(xs.map(x => x * 10), a.map(x => x * 10), xs[1..4].map(x => x * 10))
print(s.map(c => c.ord - 'a'.ord + 1), s[1..4].map(c => c.ord - 'a'.ord + 1))

// filter, partition, group, first(pred)
print(xs.filter(x => x > 1), a.filter(x => x > 1), xs[1..4].filter(x => x > 1))
print(s.filter(c => c > 'a'), s[1..4].filter(c => c > 'a'))
print(xs[1..4].partition(x => x > 1), s.partition(c => c > 'a'), s[1..4].partition(c => c > 'a'))
print(a.group(by: x => x % 2), s.group(by: c => c == 'a'), s[1..4].group(by: c => c == 'a'))
print(xs.first(x => x == 2)!, a.first(x => x == 2)!, xs[1..4].first(x => x == 2)!, s.first(c => c == 'b')!, s[1..4].first(c => c == 'b')!)

// count, all, any, contains, find, first_ge, last_le
print(xs.count(1), a.count(1), xs[1..4].count(1), s.count('a'), s[1..4].count('a'))
print(xs.count(x => x > 1), s.count(c => c > 'a'), s[1..4].count(c => c > 'a'))
print(yes(xs.all(x => x > 0)), yes(s.all(c => c >= 'a')), yes(s[1..4].all(c => c >= 'a')))
print(yes(xs.any(x => x == 2)), yes(s.any(c => c == 'b')), yes(s[1..4].any(c => c == 'b')))
print(yes(xs.contains(2)), yes(a.contains(2)), yes(s.contains('b')), yes(s[1..4].contains('b')))
print(xs.find(2)!, a.find(2)!, xs[1..4].find(2)!, s.find('b')!, s[1..4].find('b')!)

let sxs = [1, 2, 2, 4]
var sa: [0..4]Int = 0
for i in 0..4: sa[i] = sxs[i]
let ss = "abbd"
print(sxs.first_ge(2)!, sa.first_ge(2)!, sxs[1..4].first_ge(2)!, ss.first_ge('b')!, ss[1..4].first_ge('b')!)
print(sxs.last_le(2)!, sa.last_le(2)!, sxs[1..4].last_le(2)!, ss.last_le('b')!, ss[1..4].last_le('b')!)

// fold, min, max, sum
print(xs.sum(), a.sum(), xs[1..4].sum(), s.map(c => c.ord - 'a'.ord + 1).sum())
print(xs.fold((x, y) => x * 10 + y)!, a.fold((x, y) => x * 10 + y)!, xs[1..4].fold((x, y) => x * 10 + y)!)
print(s.fold(max)!, s[1..4].fold(max)!, s.min()!, s.max()!, s[1..4].min()!, s[1..4].max()!)

// sorted / sorted(by:) / sorted(desc:)
print(xs.sorted(), a.sorted(), xs[1..4].sorted(), s.sorted(), s[1..4].sorted())
print(s.sorted(by: c => -c.ord), s[1..4].sorted(desc: true))

// deduped, runs, is_sorted, windows, chunks, pairs, items (distinct covered in seq_fold_distinct.cp)
print(ss.deduped(), ss[1..4].deduped())
print(ss.runs(), ss[1..4].runs(), yes(ss.is_sorted()), yes(ss[1..4].is_sorted()))
print([w for w in s.windows(3)], [w for w in s[1..4].windows(2)], [c for c in s[1..4].chunks(2)])
print([(p, q) for (p, q) in s[1..4].pairs()], [(i, c) for (i, c) in s[1..4].items])

// In-place mutators on Str matching List / [D]T
var ms: Str = "cabac"
ms.sort()
print(ms)
ms.sort(by: c => -c.ord)
print(ms)
ms.reverse()
print(ms)
ms.swap(0, 4)
print(ms)
let popped = ms.pop()!
print(popped, ms)
ms.truncate(3)
print(ms)
ms.resize(5, 'z')
print(ms)
ms.fill('q')
print(ms)

let n: Int = read()
var b = SortedBag<Int>()
for i in 0..n:
    b.add(i * 7919 % n) // timed: bag.SortedBag.add
var s = 0
for i in 0..4 * n:
    if b.contains(i * 13 % n): s += 1 // timed: bag.SortedBag.contains
    s += b.count(i % n) // timed: bag.SortedBag.count
for i in 0..n:
    b.remove(i * 31 % n) // timed: bag.SortedBag.remove
print(s, b.len)

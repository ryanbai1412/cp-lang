// scaling: bag.SortedBag.contains
let n: Int = read()
var b = SortedBag<Int>()
for i in 0..n:
    b.add(i * 7 % n)
var s = 0
for i in 0..4 * n:
    if b.contains(i * 13 % n): s += 1 // timed
print(s)

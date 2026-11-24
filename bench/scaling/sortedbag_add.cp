// scaling: bag.SortedBag.add
let n: Int = read()
var b = SortedBag<Int>()
for i in 0..n:
    b.add(i * 7919 % n) // timed
print(b.len)

use std.bag.{SortedBag}
var win = [SortedBag<Int>(), SortedBag<Int>()]
win[0].add(1)
win[1].add(2)
win[0].absorb(win[1].take())
print(win[0].len)

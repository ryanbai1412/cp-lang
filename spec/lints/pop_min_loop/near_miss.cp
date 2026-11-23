var s = SortedBag<Int>()
for x in [3, 1, 2]:
    s.add(x)
let x = s.first!
s.remove(x + 1)
let y = s.first!
print(x, y)

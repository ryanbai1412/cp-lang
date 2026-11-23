var s = SortedBag<Int>()
for x in [3, 1, 2]:
    s.add(x)
while let x = s.pop_first():
    print(x)

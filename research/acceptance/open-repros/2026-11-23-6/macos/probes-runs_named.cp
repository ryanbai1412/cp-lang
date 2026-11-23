let xs = [3, 1, 3, 2, 1, 3, 5, 5]
let s = xs.sorted()
for r in s.runs():
    print(r.len, r[0], r.first, [x for x in r])

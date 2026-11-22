let n: 3..=100 = read()
let a: [0..n]1..=100 = read()
let odds = a.count(x => x % 2 == 1)
let want = if odds == 1: 1 else: 0
for (i, x) in a.items:
    if x % 2 == want: print(i + 1)

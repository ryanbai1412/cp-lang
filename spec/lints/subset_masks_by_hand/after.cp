let n: 1..=20 = read()
let a: [0..n]Int = read()
var best = 0
for sub in a.subsequences():
    let s = sub.items.sum()
    if s % 3 == 0:
        best max= s
print(best)

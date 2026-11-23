let n: 1..=200000 = read()
let a: [0..n]Int = read()
let order = [(a[i], i) for i in 0..n].sorted()
for (v, i) in order:
    print(i + 1, end: " ")
print()

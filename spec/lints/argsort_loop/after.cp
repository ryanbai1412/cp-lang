let n: 1..=200000 = read()
let a: [0..n]Int = read()
for i in (0..n).sorted(by: i => a[i]):
    print(i + 1, end: " ")
print()

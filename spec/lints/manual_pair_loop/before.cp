let n: 1..=2000 = read()
let a: [0..n]Int = read()
var c = 0
for i in 0..n:
    for j in i + 1..n:
        if a[i] + a[j] == 0:
            c += 1
print(c)

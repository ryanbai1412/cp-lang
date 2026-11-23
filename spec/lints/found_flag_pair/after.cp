let n: 1..=100 = read()
let a: [0..n]Int = read()
var last: ?Int = None
for i in 0..n:
    if a[i] % 2 == 0:
        last = a[i]
print(last ?? -1)

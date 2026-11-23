let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var w = Window(min)
var out: List<Int> = []
for i in 0..n:
    w.push(a[i])
    if i >= k:
        w.pop(a[i - k])
    if i >= k - 1:
        out.push(w.all!)
print(out)

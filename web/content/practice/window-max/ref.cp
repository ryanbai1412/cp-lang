let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=1e9 = read()
var w = Window(max)
var out = List<Int>()
for (i, x) in a.items:
    w.push(x)
    if w.len > k: w.pop()
    if i + 1 >= k: out.push(w.all!)
print(out)

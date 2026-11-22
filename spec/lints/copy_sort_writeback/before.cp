var xs = [5, 4, 3, 2, 1]
let sub = xs[1..4].sorted()
for i in 1..4: xs[i] = sub[i - 1]
print(xs)

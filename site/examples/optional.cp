domain V = 1..=3
let a: [V]Int = [V] v => v.idx * 10
for v in V:
    if let p = v.prev:
        print(a[v] - a[p])
    else:
        print(0)

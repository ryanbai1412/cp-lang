let (n, q): (Int, Int) = read()
domain I = 0..n
let a: [I]Int = read()
for _ in 0..q:
    let (l, r): (I, I) = read()
    var best = a[l]
    for i in l..=r:
        best max= a[i]
    print(best)

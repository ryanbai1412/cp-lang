let (n, q): (Int, Int) = read()
domain I = 1..=n
let a: [I]Int = read()
for _ in 0..q:
    let (l, r): (I, I) = read()
    var best = l
    for i in l..=r:
        if a[i] > a[best]:
            best = i
    print(best)

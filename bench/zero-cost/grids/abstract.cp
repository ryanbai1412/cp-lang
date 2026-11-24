// A max-plus product on `[S, S]Int` grids, as noi20p1's MatPow: the inner
// loop stores to `c` while it reads the sizes of grids passed by reference,
// so it must stay as fast as nested rows.
let (n, reps): (1..=512, 1..=64) = read()
domain S = 0..n
type Mat = [S, S]Int

fn mul(a: Mat, b: Mat) -> Mat:
    var c: Mat = [S, S] (i, j) => -1
    for i in S:
        for j in S:
            let via = a[i, j]
            if via >= 0:
                unchecked:
                    for l in S: c[i, l] max= via + b[j, l]
    c

var a: Mat = [S, S] (i, j) => if (i.ord * 7 + j.ord * 13) % 5 == 0: -1 else: (i.ord * 31 + j.ord * 17) % 1000
var acc = 0
for _ in 0..reps:
    a = mul(a, a)
    acc = (acc + a[S.first!, S.last!]) % 1_000_000_007
print(acc)

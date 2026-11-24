let (n, reps): (1..=512, 1..=64) = read()

fn mul(n: Int, a: List<List<Int>>, b: List<List<Int>>) -> List<List<Int>>:
    var c = [[-1 for _ in 0..n] for _ in 0..n]
    for i in 0..n:
        for j in 0..n:
            let via = a[i][j]
            if via >= 0:
                unchecked:
                    for l in 0..n: c[i][l] max= via + b[j][l]
    c

var a = [[if (i * 7 + j * 13) % 5 == 0: -1 else: (i * 31 + j * 17) % 1000 for j in 0..n] for i in 0..n]
var acc = 0
for _ in 0..reps:
    a = mul(n, a, a)
    acc = (acc + a[0][n - 1]) % 1_000_000_007
print(acc)

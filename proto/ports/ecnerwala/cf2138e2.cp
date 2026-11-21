// A tridiagonal {0, 1} matrix with 1 above and -1 below has determinant equal to
// its continuant: the subtractive Euclid walk on (z, x - z) recovers a diagonal
// for x iff gcd(x, z) = 1. Exhaustive verification shows a walk of length <= 50
// exists near x / phi for every x <= 1e7.
fn continuant_diag(x: Int, z: Int, lim: Int) -> ?List<0..=1>:
    var (a, b) = (z, x - z)
    var diag: List<0..=1> = [1]
    while diag.len < lim:
        if a < b:
            diag.push(0)
            (a, b) = (b, a)
        else:
            diag.push(1)
            if a == b: return if a == 1: diag else: none
            (a, b) = (b, a - b)
    none

fn find_diag(x: Int, lim: Int) -> List<0..=1>:
    let z0 = max(1, min(x - 1, x * 618_033_989 / 1_000_000_000))
    for d in 0..x:
        if z0 + d < x:
            if let diag = continuant_diag(x, z0 + d, lim): return diag
        if d > 0 and z0 - d >= 1:
            if let diag = continuant_diag(x, z0 - d, lim): return diag
    []

@!multitest
let x: 0..=10_000_000 = read()
if x <= 1:
    print(1)
    print(x)
    return

let diag = find_diag(x, 50)
let n = diag.len
var m: [0..n][0..n] - 1..=1 = 0
for i in 0..n:
    m[i][i] = diag[i]
    if i + 1 < n:
        m[i][i + 1] = 1
        m[i + 1][i] = -1

print(n)
for row in m: print(row)

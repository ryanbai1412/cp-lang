// A tridiagonal {0, 1} matrix with 1 above and -1 below has determinant equal to
// its continuant: the subtractive Euclid walk on (z, x - z) recovers a diagonal
// for x iff gcd(x, z) = 1. Starting z near x / phi keeps the walk under 80 steps.
fn continuant_diag(x: Int, z: Int) -> ?List<0..=1>:
    var (a, b) = (z, x - z)
    var diag: List<0..=1> = [1]
    while true:
        if a < b:
            diag.push(0)
            (a, b) = (b, a)
        else:
            diag.push(1)
            if a == b: return if a == 1: diag else: none
            (a, b) = (b, a - b)

@!multitest
let x: 0..=1e7 = read()
if x <= 1:
    print(1)
    print(x)
    return

let diag = (1..x).nearest_first(to: x / phi).find_map(z => continuant_diag(x, z).filter(d => d.len <= 80))!
let n = diag.len
var m: [0..n][0..n]-1..=1 = 0
for i in 0..n:
    m[i, i] = diag[i]
    if i + 1 < n: (m[i, i + 1], m[i + 1, i]) = (1, -1)
print(n)
for row in m: print(row)

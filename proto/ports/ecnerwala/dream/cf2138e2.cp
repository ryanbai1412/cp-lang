// Codeforces 2138 E2: Determinant Construction (Hard Version) — dream.
fn euclid_diag(x: Int, z: Int, lim: Int) -> ?SmallVec<Int, 50>:
    var a = z, b = x - z
    var diag: SmallVec<Int, 50> = [1]
    while true:
        if a < b:
            diag.push(0)
            (a, b) = (b, a)
        else:
            diag.push(1)
            if a == b: return if a == 1: diag else: none
            (a, b) = (b, a - b)
        if diag.len > lim: return none

@!multitest
let x: 0..=1e7 = read()
if x <= 1:
    print(1)
    print(x)
    return

let z0 = clamp(x * 618_033_989 / 1_000_000_000, 1, x - 1)
let diag = spiral(z0, 1..x).find_map(z => euclid_diag(x, z, 50))!
let n = diag.len
let m = [i in 0..n, j in 0..n =>
    if i == j: diag[i]
    elif j == i + 1: 1
    elif i == j + 1: -1
    else: 0]
print(n)
for row in m: print(row)

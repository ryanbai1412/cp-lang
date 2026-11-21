// Codeforces 2138 E1: Determinant Construction (Easy Version).
fn chain_len(x: Int, z: Int, lim: Int) -> ?Int:
    var a = z
    var b = x - z
    var ops = 2
    while a != b:
        if a < b:
            swap(mut a, mut b)
            ops += 1
        a -= b
        swap(mut a, mut b)
        ops += 1
        if ops > lim: return none
    if a == 1: ops else: none

fn find_start(x: Int, lim: Int) -> Int:
    let z0 = max(1, min(x - 1, x * 618_033_989 / 1_000_000_000))
    for d in 0..x:
        if z0 + d < x and chain_len(x, z0 + d, lim) != none:
            return z0 + d
        if d > 0 and z0 - d >= 1 and chain_len(x, z0 - d, lim) != none:
            return z0 - d
    1

@!multitest
let x: Int = read()
if x <= 1:
    print(1)
    print(x)
    return

let z = find_start(x, 80)
var a = z
var b = x - z
var diag = [1]
while true:
    if a < b:
        diag.push(0)
        swap(mut a, mut b)
        continue
    diag.push(1)
    if a == b: break
    a -= b
    swap(mut a, mut b)

let n = diag.len
var m: [0..n][0..n]Int = 0
for i in 0..n:
    m[i][i] = diag[i]
    if i + 1 < n:
        m[i][i + 1] = 1
        m[i + 1][i] = -1

print(n)
for i in 0..n:
    print(m[i])

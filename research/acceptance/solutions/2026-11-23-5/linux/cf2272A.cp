@!multitest
let (n, k): (2..=100, 1..=100) = read()
let a: [0..n]0..n = read()

fn value(b: List<Int>) -> Int:
    var seen = [n + 1].filled(false)
    var mex = 0
    var f: List<Int> = []
    for x in b:
        seen[x] = true
        while seen[mex]: mex += 1
        f.push(mex)
    return f[n - k]

let base = value([x for x in a])
var best = base
var pick = (-1, -1)
for i in 0..n:
    for j in i + 1..n:
        var b = [x for x in a]
        (b[i], b[j]) = (b[j], b[i])
        let v = value(b)
        if v > best:
            best = v
            pick = (i, j)
if pick.0 < 0: print(-1)
else: print(pick.0 + 1, pick.1 + 1)

@check
fn judge():
    let b0 = value([x for x in a])
    var top = b0
    for i in 0..n:
        for j in i + 1..n:
            var b = [x for x in a]
            (b[i], b[j]) = (b[j], b[i])
            top = max(top, value(b))
    let first: Int = read_output()
    if first == -1:
        assert b0 == top
    else:
        let second: Int = read_output()
        assert 1 <= first and first <= n and 1 <= second and second <= n and first != second
        var b = [x for x in a]
        (b[first - 1], b[second - 1]) = (b[second - 1], b[first - 1])
        assert value(b) == top

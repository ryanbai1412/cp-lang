@!multitest
let (n, k): (2..=100, 1..=n) = read()
let a: [0..n]0..n = read()

fn value(b: List<Int>) -> Int:
    var f: List<Int> = []
    var seen = [false for _ in 0..n + 1]
    var mex = 0
    for x in b:
        seen[x] = true
        while seen[mex]: mex += 1
        f.push(mex)
    f.sorted(by: desc)[k - 1]

let base = value(a)
var best = base
var pick: ?(Int, Int) = None
for i in 0..n:
    for j in i + 1..n:
        var b = a.clone()
        b.swap(i, j)
        let v = value(b)
        if v > best:
            best = v
            pick = Some((i + 1, j + 1))
if pick is Some((i, j)):
    print(i, j)
else:
    print(-1)

@check
fn judge():
    let b0 = value(a.clone())
    var opt = b0
    for i in 0..n:
        for j in i + 1..n:
            var b = a.clone()
            b.swap(i, j)
            opt max= value(b)
    let first: Int = read_output()
    if first == -1:
        assert opt == b0
    else:
        let second: Int = read_output()
        assert 1 <= first and first <= n and 1 <= second and second <= n and first != second
        var b = a.clone()
        b.swap(first - 1, second - 1)
        assert value(b) == opt and opt > b0

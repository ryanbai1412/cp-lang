@!multitest
let n: 2..=100 = read()
let k: 1..=n = read()
let a: [0..n]0..n = read()

fn kth(b: List<Int>, n: Int, k: Int) -> Int:
    var seen = [false for _ in 0..=n]
    var m = 0
    var f: List<Int> = []
    for x in b:
        seen[x] = true
        while seen[m]: m += 1
        f.push(m)
    f.sorted(by: desc)[k - 1]

let base = kth([x for x in a], n, k)
var best = base
var (bi, bj) = (-1, -1)
for i in 0..n:
    for j in i + 1..n:
        var b = [x for x in a]
        (b[i], b[j]) = (b[j], b[i])
        let v = kth(b, n, k)
        if v > best:
            best = v
            (bi, bj) = (i, j)
if bi < 0:
    print(-1)
else:
    print(bi + 1, bj + 1)

@check
fn check():
    fn val(b: List<Int>) -> Int:
        var seen = [false for _ in 0..=n]
        var m = 0
        var f: List<Int> = []
        for x in b:
            seen[x] = true
            while seen[m]: m += 1
            f.push(m)
        f.sorted(by: desc)[k - 1]
    var opt = val([x for x in a])
    for i in 0..n:
        for j in i + 1..n:
            var b = [x for x in a]
            (b[i], b[j]) = (b[j], b[i])
            opt = max(opt, val(b))
    let x: Int = read_output()
    var b = [x for x in a]
    if x != -1:
        let y: Int = read_output()
        assert 1 <= x and x <= n and 1 <= y and y <= n and x != y
        (b[x - 1], b[y - 1]) = (b[y - 1], b[x - 1])
    assert val(b) == opt

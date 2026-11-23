@!multitest
let (n, k): (2..=100, 1..=100) = read()
let a: [0..n]0..n = read()

fn kth(b: List<Int>) -> Int:
    var seen = [false for _ in 0..=n]
    var m = 0
    var f: List<Int> = []
    for x in b:
        seen[x] = true
        while seen[m]:
            m += 1
        f.push(m)
    f.sorted(by: desc)[k - 1]

let base = kth([x for x in a])
var best = base
var ans = (-1, -1)
var b = [x for x in a]
for i in 0..n:
    for j in i + 1..n:
        (b[i], b[j]) = (b[j], b[i])
        let v = kth(b)
        if v > best:
            best = v
            ans = (i + 1, j + 1)
        (b[i], b[j]) = (b[j], b[i])
if ans.0 == -1:
    print(-1)
else:
    print(ans.0, ans.1)

@check
fn judge():
    let start = kth([v for v in a])
    var opt = start
    var c = [v for v in a]
    for i in 0..n:
        for j in i + 1..n:
            (c[i], c[j]) = (c[j], c[i])
            opt = max(opt, kth(c))
            (c[i], c[j]) = (c[j], c[i])
    let x: Int = read_output()
    if x == -1:
        assert opt == start
    else:
        let y: Int = read_output()
        (c[x - 1], c[y - 1]) = (c[y - 1], c[x - 1])
        assert x != y and kth(c) == opt

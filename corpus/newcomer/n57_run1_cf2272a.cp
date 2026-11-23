// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2272A, timed round.
@!multitest
let (n, k): (2..=100, 1..=100) = read()
let a: [0..n]0..n = read()
let p = n - k + 1
var seen: [0..=n]Bool = false
for i in 0..p:
    seen[a[i]] = true
var m = 0
while seen[m]:  // expect: manual_mex
    m += 1
if m == p:
    print(-1)
else:
    var j = 0
    for i in 0..p if a[i] > a[j]:  // expect: loop_argmax
        j = i
    let q = (0..n).find(i => a[i] == m)!
    print(j + 1, q + 1)

@check
fn valid():
    let p = n - k + 1
    fn mex_at(b: List<Int>) -> Int:
        var s: [0..=n]Bool = false
        for i in 0..p:
            s[b[i]] = true
        var x = 0
        while s[x]:
            x += 1
        return x
    var best = mex_at([x for x in a])  // expect: copy_comprehension
    for i in 0..n:
        for j in i + 1..n:
            var b = [x for x in a]
            b.swap(i, j)
            best max= mex_at(b)
    let first: Int = read_output()
    if first == -1:
        assert mex_at([x for x in a]) == best
    else:
        let second: Int = read_output()
        var b = [x for x in a]
        b.swap(first - 1, second - 1)
        assert first != second and mex_at(b) == best

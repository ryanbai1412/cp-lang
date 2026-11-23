// A. Maximize One MEX
// https://codeforces.com/contest/2272/problem/A
// f is nondecreasing, so the k-th largest is f[m] with m = n - k + 1; bring the smallest value missing
// from the first m into it, swapping out a value >= m
@!multitest
let (n, k): (2..=100, 1..=n) = read()
let a: [0..n]0..n = read()
let m = n - k + 1

fn prefix_mex(b: [0..n]0..n, len: Int) -> Int:
    var seen: [0..=n]Bool = false
    for i in 0..len: seen[b[i]] = true
    (0..=n).first(v => not seen[v])!

var seen: [0..n]Bool = false
for i in 0..m: seen[a[i]] = true
if let x = (0..m).first(v => not seen[v]):
    let i = (0..m).first(i => a[i] >= m)!
    let j = a.find(x)!
    print(i + 1, j + 1)
else:
    print(-1)

@check
fn check():
    let len = n - k + 1
    let x: Int = read_output()
    if x == -1:
        assert prefix_mex(a, len) == len
        return
    let y: Int = read_output()
    var b = a.copy()
    b.swap(x - 1, y - 1)
    var best = prefix_mex(a, len)
    for i in 0..n:
        for j in i + 1..n:
            var c = a.copy()
            c.swap(i, j)
            best = max(best, prefix_mex(c, len))
    assert prefix_mex(b, len) == best

// A. Composite Permutation
// https://codeforces.com/contest/2285/problem/A
@!multitest
let n: 1..=100 = read()
if n < 4:
    print(-1)
    return
print([if i == 1: 4 elif i == 4: 1 else: i for i in 1..=n])

fn composite(x: Int) -> Bool = x >= 4 and (2..x).any(d => d * d <= x and x % d == 0)

@check
fn check():
    if n < 4:
        let x: Int = read_output()
        assert x == -1
        return
    let p: [0..n]1..=n = read_output()
    assert p.distinct().len == n
    for (i, x) in p.items:
        assert composite((i + 1) * x)

@!multitest
let n: 1..=100 = read()
if n < 4:
    print(-1)
else:
    var p = [i for i in 1..=n]
    p[0] = 4
    p[3] = 1
    print(p)

@check
fn judge():
    if n < 4:
        let k: Int = read_output()
        assert k == -1
    else:
        let p: [0..n]1..=n = read_output()
        var seen: Set<Int> = Set()
        for (i, x) in p.items:
            seen.add(x)
            let v = (i + 1) * x
            assert v >= 4 and (2..v).any(d => v % d == 0)
        assert seen.len == n

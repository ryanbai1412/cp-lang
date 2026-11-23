use std.math.{is_prime}

@!multitest
let n: 1..=100 = read()
if n < 4:
    print(-1)
    return
var p = [i for i in 1..=n]
(p[0], p[3]) = (p[3], p[0])
print(p)

@check
fn check():
    if n < 4:
        let k: Int = read_output()
        assert k == -1
        return
    let q: [0..n]1..=n = read_output()
    assert q.distinct().len == n
    for i in 0..n:
        let v = (i + 1) * q[i]
        assert v > 1 and not is_prime(v)

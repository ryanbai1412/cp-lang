@!multitest
let n: 1..=100 = read()
if n < 4:
    print(-1)
    return
print([4, 2, 3, 1] + [i for i in 5..=n])

fn composite(x: Int) -> Bool = x >= 4 and (2..x).any(d => d * d <= x and x % d == 0)

@check
fn judge():
    if n < 4:
        let k: Int = read_output()
        assert k == -1
    else:
        let p: [0..n]1..=n = read_output()
        assert p.sorted() == [i for i in 1..=n]
        for i in 0..n:
            assert composite((i + 1) * p[i])

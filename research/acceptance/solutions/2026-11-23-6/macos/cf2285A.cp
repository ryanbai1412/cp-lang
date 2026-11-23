@!multitest
let n: 1..=100 = read()
if n <= 3:
    print(-1)
else:
    var p = [i for i in 1..=n]
    p.swap(0, 3)
    print(p)

fn composite(x: Int) -> Bool = x >= 4 and (2..x).any(d => x % d == 0)

@check
fn judge():
    if n <= 3:
        let k: Int = read_output()
        assert k == -1
    else:
        let p: [0..n]1..=100 = read_output()
        assert p.sorted() == [i for i in 1..=n]
        for (i, x) in p.items:
            assert composite((i + 1) * x)

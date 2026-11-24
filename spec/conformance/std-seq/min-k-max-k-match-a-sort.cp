// expect: ok, output "ok"
// rule: 08.1.2
// `min_k`/`max_k` keep a buffer of at most `2k` and cut it back with `select`; every `k`, ties
// included, gives what a stable sort's first `k` gives.
var xs: List<(Int, Int)> = []
var s = 7
for i in 0..300:
    s = (s * 1103515245 + 12345) % 2147483648
    xs.push((s % 17, i))
for k in [0, 1, 2, 5, 16, 17, 150, 299, 300, 301]:
    let want_lo = xs.sorted(by: p => p.0)[0..min(k, xs.len)].to_list()
    let want_hi = xs.sorted(by: p => -p.0)[0..min(k, xs.len)].to_list()
    assert xs.min_k(k, by: p => p.0).sorted(by: p => (p.0, p.1)) == want_lo.sorted(by: p => (p.0, p.1))
    assert xs.max_k(k, by: p => p.0).sorted(by: p => (p.0, p.1)) == want_hi.sorted(by: p => (p.0, p.1))
print("ok")

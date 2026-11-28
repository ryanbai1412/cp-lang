// expect: E0902
// rule: R03.1.10
// `m` is at least 1, so `0` is never in `m..10`.
fn g(m: 1..=5) -> Int:
    let r: m..10 = 0
    r
print(g(3))

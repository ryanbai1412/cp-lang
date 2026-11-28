// expect: E0902
// rule: R03.1.10
// A range type written in a body with a value end holds each end in a slot that keeps what
// it can be: `m` is at most 5, so `9` is never in `0..m`.
fn g(m: 1..=5) -> Int:
    let r: 0..m = 9
    r
print(g(3))

// expect: R0201
// rule: R03.1.10 R02.10.2
// `2` is in `0..m` for every `m` in `4..=9`, so it needs no check; `k` may not be, so it is
// checked when stored. `h` is checked before `g`, and what its slots held stays in `h`.
fn h(m: 1..=9, k: Int) -> Int:
    let r: 0..m = k
    r
fn g(m: 4..=9) -> Int:
    let r: 0..m = 2
    let s: 0..=m = 4
    r + s
print(h(3, 2), g(5))
print(h(3, 3))

// expect: E0305 at 4
// rule: 07.4.7 07.4.5
// A reversed slice walks out of key order, so it has no `[]`, even held in a local.
fn last(xs: List<Int>) -> Int:
    let b = xs[1..].rev
    b[0]
print(last([1, 2, 3]))

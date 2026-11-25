// expect: E0608
// rule: R05 R2
// A move inside a loop moves the same binding on every round.
fn go(k: Int) -> Int:
    let a = [1, 2, 3]
    var n = 0
    for i in 0..k:
        let b = a  // <- here
        n += b.len
    n
print(go(2))

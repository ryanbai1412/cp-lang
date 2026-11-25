// expect: R0201
// rule: R02.10.2, R05.10.1
// input: "4\n"
// A run-time range written through a closure that shares the var (R05.10.1 + R02.10.2).
fn go(n: Int) -> Int:
    var i: 1..=n = 1
    let bump = () => i += 1  // <- here
    for k in 0..n: bump()
    i
print(go(read()))

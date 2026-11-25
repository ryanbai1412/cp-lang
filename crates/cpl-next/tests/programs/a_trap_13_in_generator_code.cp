// expect: R0202
// rule: R09.3
// Trap inside a generator's own code, reported where it is written.
fn gen(n: Int) -> yields Int:
    var x = 4611686018427387904
    for i in 0..n:
        x = x + x  // <- here
        yield x
for v in gen(2): print(v)

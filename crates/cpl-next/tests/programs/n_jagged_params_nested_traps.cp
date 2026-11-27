// expect: R0201
// rule: R06.4.1
// A nested fn's jagged parameter is checked at the call too, against the earlier argument.
fn run(m: 0..=9) -> Int:
    fn h(a: 0..=9, b: 0..a) -> Int = a - b
    print(h(m, m - 1))
    h(m, m)  // <- here
print(run(4))

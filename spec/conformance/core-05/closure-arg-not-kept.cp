// expect: E0603
// rule: 05.10.4
fn keep(f: (Int) -> Int, mut out: List<(Int) -> Int>): out.push(f) // <- here

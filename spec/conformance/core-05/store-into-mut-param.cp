// expect: E0603
// rule: 05.7.9
fn keep(mut out: List<Str>, s: Str): out.push(s) // <- here

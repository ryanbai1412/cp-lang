// expect: R0201 (checked)
// rule: 04.4.24
// A list literal given for a `List<lo..=hi>` parameter checks each cell as a scalar argument is checked, so the body may rely on the range.
fn first(xs: List<-5..=5>) -> Int = xs[0]
fn neg(v: Int) -> Int = -v
let a: Int = 3
print(first([neg(a) * 3])) // <- here

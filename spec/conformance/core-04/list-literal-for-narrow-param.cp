// expect: ok, output "3 -3"
// rule: 04.4.24
// A list literal given for a narrow list parameter is built in the parameter's cells, whether its cells are `Int` or already narrow.
fn first(xs: List<-5..=5>) -> Int = xs[0]
fn neg(v: Int) -> Int = -v
let a: 0..=5 = 3
let low: -5..=5 = -a
print(first([-neg(a)]), first([low]))

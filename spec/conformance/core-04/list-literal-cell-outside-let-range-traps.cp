// expect: R0201 (checked)
// rule: 04.4.24
// A list literal bound to a declared `List<lo..=hi>` checks each cell, as a push does.
fn neg(v: Int) -> Int = -v
let a: Int = 3
let ys: List<-5..=5> = [neg(a) * 3] // <- here
print(ys[0])

// expect: E0301
// rule: R04.3.1
// A position of one domain never indexes an array over another, though both have three.
domain Row = 1..=3
domain Col = 1..=3
let a: [Row]Int = 0
print(a[Col.first!]) // <- here

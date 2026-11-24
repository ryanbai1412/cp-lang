// expect: E0302
// rule: 02.4.4
let a: ?Int = Some(1)
let b: Int = a + 1 // <- here
print(b)

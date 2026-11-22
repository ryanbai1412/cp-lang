// expect: E0302 at 2
// rule: 02.4.4
let a: ?Int = Some(2)
let b: Int = a + 1
print(b)

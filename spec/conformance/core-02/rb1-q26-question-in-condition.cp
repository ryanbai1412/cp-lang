// expect: E0803
// rule: 02.4.7
let x: ?Int = Some(3)
if x? > 2: print(1) // <- here

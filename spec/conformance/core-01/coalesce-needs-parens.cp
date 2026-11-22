// expect: E0110 at 2
// rule: 01.5.6
let a: ?Int = Some(1)
print(a ?? 0 + 1)

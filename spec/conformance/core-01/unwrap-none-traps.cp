// expect: R0301 (checked)
// rule: 01.8.6
let x: ?Int = None
print(x! + 1) // <- here

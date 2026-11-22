// expect: ok, output "None 5"
// rule: 02.4.4
let a: ?Int = None
let b: ?Int = 4
print(a + 1, b! + 1, none: "None")

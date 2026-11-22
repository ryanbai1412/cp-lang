// expect: ok, output "6"
// rule: 01.6.2
let x: ?Int = 5
let Some(y) = x else: return
print(y + 1)

// expect: ok, output "1"
// engines: next
let a: ?Int = Some(1)
let b: ?Int = None
print(if a == Some(1) and a != b: 1 else: 0)

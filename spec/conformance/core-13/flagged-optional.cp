// expect: ok, output "0 18446744073709551615"
// rule: 13.2.6
let x: ?U64 = None
let y: ?U64 = Some(18446744073709551615)
print(x ?? 0, y!)

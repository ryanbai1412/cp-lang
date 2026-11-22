// expect: ok, output "x"
// rule: 02.4.5
let d: ?Int = None
if (d ?? 0) + 1 < 5: print("x")

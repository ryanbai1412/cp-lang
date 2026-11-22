// expect: ok, output "1"
// rule: 01.3.6
enum Side = Left | None
let s = Side.None
print(if s == Side.None: 1 else: 0)

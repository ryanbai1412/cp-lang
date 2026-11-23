// expect: ok, output "7"
// rule: 10.3.7
// A `?Int` prints through a fallback.
let x: ?Int = None
print(x ?? 7)

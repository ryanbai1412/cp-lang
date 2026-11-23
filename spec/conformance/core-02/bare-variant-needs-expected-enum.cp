// expect: E0201 at 4
// rule: 02.7.1
// A bare variant resolves only where the expected type is its enum; `Left` of two enums with no annotation is an error, not a pick.
enum Side = Left | Right
enum Turn = Left | Back
let s = Left
print(1)

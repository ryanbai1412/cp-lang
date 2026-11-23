// expect: ok, output "Back Left"
// rule: 02.7.1
// Where the expected type is an enum, a bare variant that two enums declare is that enum's: a `let` type, a return, a parameter, a list's element type.
enum Turn = Left | Back
enum Side = Left | Right
fn flip(x: Turn) -> Turn = if x == Back: Left else: Back
let t: Turn = Left
let sides: List<Side> = [Left, Right]
print(flip(t), sides[0])

// A bare variant two enums declare reads as the expected enum (R02.7.1):
// in a `let` type, a return, an argument, a field, `=`, and a list literal.
enum Turn = Left | Back
enum Side = Left | Right
struct P:
    t: Turn
fn flip(x: Turn) -> Turn = if x == Back: Left else: Back
fn first() -> Turn:
    return Left
var cur: Turn = Back
cur = Left
var ts: List<Turn> = []
ts.push(Left)
let p = P(t: Left)
let t: Turn = Left
let sides: List<Side> = [Left, Right]
for x in [flip(t), first(), cur, ts[0], p.t, flip(Left), flip(Back)]:
    match x:
        Left => print("left")
        Back => print("back")
print(yes(cur == Left), yes(sides[0] == Left), sides.len)

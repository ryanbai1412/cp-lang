// expect: ok, output "1 0 1 0 1"
// rule: R02.7
// `E(x)` of a payload-free enum is the variant at position `x`, and `Int(e)` its position.
enum Color:
    Red
    Green
enum Dir:
    L
    R
let g = Color(1)
print(Int(g), Int(Color(0)), Int(Dir(1)), Int(Color.Red), Int(Color(Int(Color.Green))))

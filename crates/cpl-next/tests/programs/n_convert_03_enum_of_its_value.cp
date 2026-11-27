// expect: ok, output "5 2 1 2 5"
// rule: R02.7
// `E(x)` of a payload-free enum is the variant whose value is `x`, and `Int(e)` its value.
enum Color:
    Red = 2
    Green = 5
enum Dir:
    L
    R
let g = Color(5)
print(Int(g), Int(Color(2)), Int(Dir(1)), Int(Color.Red), Int(Color(Int(Color.Green))))

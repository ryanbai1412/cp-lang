// expect: ok, output "10\n4"
// rule: 01.6.2
enum Shape:
    Circle(r: Int)
    Square(s: Int)
let x: ?Int = 5
let (Some(w), k) = (x, 2) else: return
print(w * k)
let sh = Shape.Circle(3)
let Shape.Circle(r) = sh else: return
print(r + 1)

// expect: ok, output "12 12"
// rule: 08.5.4
// Patterns are the stored shape: enum variants and their fields.
enum Shape = Circle(r: Int) | Rect(w: Int, h: Int)
fn area(s: Shape) -> Int = match s:
    Circle(r) => 3 * r * r
    Rect(w, h) => w * h
print(area(Circle(2)), area(Rect(3, 4)))

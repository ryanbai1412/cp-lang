// expect: ok, output "12 3"
// rule: 13.2.8
enum Sh:
    Circle(r: Int)
    Rect(w: Int, h: Int)
fn size(s: Sh) -> Int = match s:
    Circle(r) => r
    Rect(w, h) => w * h
print(size(Rect(3, 4)), size(Circle(3)))

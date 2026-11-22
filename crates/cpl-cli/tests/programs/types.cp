// Structs with methods, enums with and without payloads, records,
// tuples and optionals.
struct Point:
    var x: Int
    var y: Int

    fn dist(self, other: Point) -> Int:
        abs(self.x - other.x) + abs(self.y - other.y)

    fn origin() -> Point:
        Point(x: 0, y: 0)

    fn shift(mut self, dx: Int):
        self.x += dx

enum Color:
    Red
    Green
    Blue

enum Shape:
    Circle(Int)
    Rect(Int, Int)
    Dot

fn area(s: Shape) -> Int:
    match s:
        Circle(r) => 3 * r * r
        Rect(w, h) => w * h
        Dot => 0

fn name(c: Color) -> Str:
    match c:
        Red => "red"
        Green => "green"
        Blue => "blue"

let p = Point(x: 3, y: 4)
var q = Point.origin()
print(p, q, p.dist(q), yes(p == q), yes(p < q))
q.shift(5)
q.y = -2
print(q, p.dist(q))
print(Color.Green, name(Color.Blue), yes(Color.Red == Color.Red), yes(Color.Red != Color.Blue))
let shapes: List<Shape> = [Shape.Circle(2), Shape.Rect(3, 4), Shape.Dot]
for s in shapes:
    print(s, area(s))
print(shapes)
let pair = (1, "one")
let (num, word) = pair
print(pair, num, word, pair.0, pair.1)
let rec = (id: 7, tag: 'x')
print(rec, rec.id, rec.tag)
var maybe: ?Int = none
print(if let v = maybe: "{v}" else: "none", maybe ?? 42, yes(maybe.is_none))
maybe = 5
print(if let v = maybe: "{v}" else: "none", maybe ?? 42, yes(maybe.is_some))
if let v = maybe:
    print("holds", v)
var points: List<Point> = []
for i in 0..4:
    points.push(Point(x: 3 - i, y: i * i))
points.sort(by: pt => pt.x)
print(points)
points[0].shift(100)
points[1].y += 7
print(points[0], points[1])
var best: ?Point = none
for pt in points:
    if pt.y > 3:
        best = pt
print(best!)

// expect: ok, output "4 6"
// rule: 08.1.5
struct V2: x: Int, y: Int
fn (a: V2) + (b: V2) -> V2 = V2(a.x + b.x, a.y + b.y)
var p = V2(1, 2)
p += V2(3, 4)
print(p.x, p.y)

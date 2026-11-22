// expect: ok, output "5"
// rule: 01.1.3
struct P: x: Int, y: Int
fn dist(a: P, b: P) -> Int:
    abs(a.x - b.x) + abs(a.y - b.y)
print(dist(P(1, 2), P(4, 0)))

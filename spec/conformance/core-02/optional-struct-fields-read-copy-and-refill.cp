// expect: ok, output "3\n1 10\nnone\n6\nnone\n3 -1"
// rule: 02.4.2
// A struct inside an optional reads through `o!.f`, copies out with `with`, empties and refills, and passes as `?P`.
struct P(x: Int, y: Int)
var o: ?P = P(1, 2)
print(o!.x + o!.y)
let q = o! with x: 10
print(o!.x, q.x)
o = None
if let p = o:
    print(p.x)
else:
    print("none")
o = P(3, 4)
let ps: List<?P> = [P(5, 6), None]
for p in ps:
    if let v = p:
        print(v.y)
    else:
        print("none")
fn sx(p: ?P) -> Int:
    if let v = p:
        return v.x
    -1
print(sx(o), sx(None))

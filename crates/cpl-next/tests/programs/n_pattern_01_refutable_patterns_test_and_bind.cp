// expect: ok, output "7 0\n9 2 0\n4\n-1 0 1 5\n10 2 -9\n4 9\n6"
// Refutable patterns (R01.7): `Some(p)`, nested tuples and variants in `let ... else`, `if let`
// and `while let` on an optional's value, `match` on ints, tuples and optionals, guards,
// or-patterns, and an arm that leaves.
enum Shape:
    Circle(r: Int)
    Rect(w: Int, h: Int)
fn opt(x: ?Int) -> Int:
    let Some(y) = x else: return 0
    y
fn pair(x: ?Int, k: Int) -> Int:
    let (Some(w), 2) = (x, k) else: return 0
    w
fn radius(s: Shape) -> Int:
    let Shape.Circle(r) = s else: return 0
    r
fn sign(n: Int) -> Int = match n:
    0 => 0
    1 | 2 | 3 => 1
    _ if n < 0 => -1
    _ => 5
fn size(p: (Int, ?Int)) -> Int = match p:
    (0, _) => 0
    (_, None) => -9
    (a, Some(b)) => if a + b > 4: 10 else: a + b
fn first_even(xs: List<Int>) -> Int:
    var i = 0
    while i < xs.len:
        let found = match xs[i] % 2:
            0 => xs[i]
            _ =>
                i += 1
                continue
        return found
    -1
print(opt(Some(7)), opt(None) + pair(None, 2))
print(pair(Some(9), 2), radius(Shape.Circle(2)), radius(Shape.Rect(1, 2)))
let q: ?(Int, Int) = Some((1, 3))
if let (a, b) = q: print(a + b)
print(sign(-4), sign(0), sign(2), sign(9))
let b: ?Int = Some(4)
let c: ?Int = None
print(size((1, b)), size((1, Some(1))), size((3, c)))
var xs = [(1, 3), (2, 2)]
var t = 0
while let (k, v) = xs.pop(): t += k * v
print(first_even([3, 4]), t + 2)
print(first_even([1, 6]))

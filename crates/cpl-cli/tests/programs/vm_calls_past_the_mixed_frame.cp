// Int calls the VM's mixed frame can't take run on the VM too: nine int
// slots of args, and a `mut` optional place it can't move out.
struct P3: x: Int, y: Int, z: Int
fn vol(a: P3, b: P3, c: P3) -> Int:
    var s = 0
    for i in 0..a.x: s += b.y * c.z + i
    s
fn bump(mut o: ?Int) -> Int:
    o = Some((o ?? 0) + 1)
    o ?? 0
struct Box: v: ?Int
let k: Int = read()
let a = P3(x: 1, y: 2, z: k)
var slots: List<?Int> = [None, Some(4)]
var bx = Box(v: None)
let x1: Int = vol(a, a, a) + 1
let x2: Int = bump(mut slots[0]) + bump(mut slots[k - 2])
let x3: Int = bump(mut bx.v) + 1
let nine = (p: Int, q: Int, r: Int, s: Int, t: Int, u: Int, v: Int, w: Int, x: Int) =>
    p + q + r + s + t + u + v + w + x * k
let x5: Int = nine(1, 2, 3, 4, 5, 6, 7, 8, 9) + 1
print(x1, x2, x3, x5, slots, bx.v, none: 0)

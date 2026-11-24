// A user's lowercase `none` is an ordinary name; `None` stays the literal beside it.
struct G:
    a: List<List<Int>>
    b: ?Int
fn mk() -> G:
    let none: List<Int> = [7]
    G([none, none], None)
fn f(b: ?Int, a: List<Int>) -> Int = a.len + (if let x = b: x else: 10)
fn h() -> Bool:
    let none: ?Int = None
    none == None
let g = mk()
let none = [1, 2]
print(g.a.len, g.a[0][0], yes(g.b == None), f(None, none), yes(h()))

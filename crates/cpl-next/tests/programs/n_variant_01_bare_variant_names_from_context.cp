// expect: ok, output "1 0\n7 3\n2 -1\n5"
// Bare variant names (R02.7.1): the one enum in scope with that variant, the expected enum
// through an annotation and the arms of an `if`, and a variant that shadows a library name.
enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
    fn is_query(self) -> Bool = self is Ask
enum Side = Left | Right
fn width(e: Ev) -> Int =
    match e:
        Add(v) => v
        Ask(l, r) => r - l
let e = Ask(1, 2)
let f = Add(5)
print(if e.is_query(): 1 else: 0, if f.is_query(): 1 else: 0)
print(width(Add(7)), width(Ask(2, 5)))
let k = 3
let (c, s): (Int, Side) = if k > 2: (2, Left) else: (1, Right)
print(c, match s:
    Left => -1
    Right => 1)
let g: Ev = Add(5)
match g:
    Add(v) => print(v)
    Ask(_, _) => print(0)

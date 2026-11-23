// expect: ok, output "3 2 0\n5"
// A library domain `S` with a `len` method, beside a struct `S_dom` with one: in C++ the domain's statics take names no struct or fn has.
struct S():
    x: Int
    @get fn len(self) -> Int = 2
impl Index for S:
    let count = 3
    @get fn ord(self) -> Int = x
    fn nth(k: Int) -> S = S(k)
struct S_dom():
    y: Int
    @get fn len(self) -> Int = 5
let a: [S]Int = 0
print(S.count, S(1).len, a[S(2)])
print(S_dom(0).len)

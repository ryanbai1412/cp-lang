// expect: E0602
// (spec) R10 (a function value's effect is every closure it can hold; reviewer and today's compiler accept: QUESTION)
// The body calls a function value whose set has a member that writes the walked global.
var xs: List<Int> = [1, 2, 3]
fn pick(b: Bool) -> Int -> ():
    if b: return (v: Int) => xs.push(v)
    (v: Int) => ()
let f = pick(xs.len > 5)
var s = 0
for x in xs:
    f(x)  // <- here
    s += x
print(s, xs.len)

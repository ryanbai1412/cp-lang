// expect: E0405
// False-rejection probe: generator reads field `a` of a global struct, body writes field `b`.
struct St:
    a: List<Int>
    b: Int
var st = St([1, 2, 3], 0)
fn walk() -> yields Int:
    for x in st.a: yield x
for x in walk(): st.b += x  // <- here
print(st.b)

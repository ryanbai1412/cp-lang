// expect: E0405
// A method generator over one field; the body writes another field of the same local struct.
struct St:
    a: List<Int>
    b: Int
    fn items(self) -> yields Int:
        for x in a: yield x
fn go() -> Int:
    var st = St([1, 2, 3], 0)
    for x in st.items(): st.b += x  // <- here
    st.b
print(go())

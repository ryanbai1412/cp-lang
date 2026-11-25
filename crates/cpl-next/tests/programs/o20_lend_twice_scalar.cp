// expect: E0601
// The same Int lent twice for writing: must be an error.
fn two(mut a: Int, mut b: Int):
    a += 1
    b += 10
fn go() -> Int:
    var x = 0
    two(mut x, mut x)  // <- here
    x
print(go())

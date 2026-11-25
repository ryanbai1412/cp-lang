// expect: ok, output "21"
// Legal: reading the walked list inside the body.
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    var s = 0
    for x in xs:
        s += x * xs.len + xs[0]
    s
print(go())

// expect: E0603
// (spec) R5
// The generator's argument is a call result (a temporary); body writes the source list.
fn ident(xs: List<Int>) -> List<Int>:
    xs  // <- here
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    for x in ident(xs): xs.push(x)
    xs.len
print(go())

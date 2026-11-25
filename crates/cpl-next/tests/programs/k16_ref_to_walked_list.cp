// expect: E0602
// (spec) R12
// Unsound if accepted: the closure pushes to the list being walked, through a shared reference.
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    let grow = () => xs.push(7)
    var s = 0
    for x in xs:
        if xs.len < 40: grow()  // <- here
        s += x
    s * 100 + xs.len
print(go())

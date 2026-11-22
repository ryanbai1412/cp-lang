// expect: E0312 at 5
var x = 0
fn bump() -> Int:
    x += 1
    return x
debug(bump())

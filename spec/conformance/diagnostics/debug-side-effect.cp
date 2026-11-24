// expect: E0312
var x = 0
fn bump() -> Int:
    x += 1
    return x
debug(bump()) // <- here

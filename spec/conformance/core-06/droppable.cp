// expect: W0101
// rule: 06.4.8
@droppable
fn f() -> Int = 1
fn g() -> Int = 2
f()
g() // <- here

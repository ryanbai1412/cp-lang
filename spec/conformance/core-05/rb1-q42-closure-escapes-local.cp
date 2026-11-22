// expect: E0603 at 3
// rule: 05.10.2
fn counter() -> (() -> Int):
    var c = 0
    () => c

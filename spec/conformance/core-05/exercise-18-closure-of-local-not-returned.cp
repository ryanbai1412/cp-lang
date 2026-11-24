// expect: E0603
// rule: 05.7.5
fn counter() -> (() -> Int):
    var c = 0
    () => c // <- here

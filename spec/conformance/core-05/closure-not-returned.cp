// expect: E0603
// rule: 05.7.5 05.10.2 06.2.2
fn counter() -> (() -> Int):
    var c = 0
    () => c // <- here

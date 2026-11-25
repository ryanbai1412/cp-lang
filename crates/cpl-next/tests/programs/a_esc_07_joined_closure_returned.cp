// expect: E0603
// rule: R05.7.8
// if-joined closures, one of which borrows a local var, returned.
fn make(k: Int) -> () -> Int:
    var c = 5
    let f = if k > 0: () => { c += 1; c } else: () => 7
    f  // <- here
let h = make(1)
print(h())

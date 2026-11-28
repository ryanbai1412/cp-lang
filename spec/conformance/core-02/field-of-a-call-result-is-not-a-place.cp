// expect: E0401
// rule: 02.6.1
// A field of a call's result names no storage, so assigning it writes nothing and is an error.
struct N:
    var cnt: Int
fn mk() -> N = N(1)
mk().cnt = 5 // <- here

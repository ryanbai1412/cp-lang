// expect: E0503
// rule: 01.3.1
// tool: check
// has: "`some` is spelled `Some`"
let a: ?Int = Some(3)
match a:
    some(x) => print(x) // <- here
    None => print(0)

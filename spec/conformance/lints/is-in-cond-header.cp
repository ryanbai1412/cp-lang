// expect: L0207
// rule: 01.7.1
let x: ?Int = Some(3)
if x is Some(v) and v > 0: // <- here
    print(v)
